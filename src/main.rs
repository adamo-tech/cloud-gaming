mod capture;
mod cursor;
mod emulator;
mod gamepad;
mod keyboard;
mod mouse;

use std::collections::HashSet;
use std::error::Error;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, mpsc};
use std::time::Duration;

use adamo::p2p::P2pOptions;
use adamo::{AudioChannels, AudioOptions, ControlMessage, Robot, VideoOptions};
use clap::Parser;

const AUDIO_SINK: &str = "cloud_gaming_audio";
const MONITOR_CAPTURE: &str = "\"props,stream.capture.sink=true\"";
const GAMESCOPE_INPUT_SOCKET: &str = "gamescope-0-ei";

#[derive(Parser)]
struct Args {
    #[arg(long, default_value = "cloud-gaming")]
    name: String,
    #[arg(long, default_value_t = 2560)]
    max_width: u32,
    #[arg(long, default_value_t = 1440)]
    max_height: u32,
    #[arg(long, default_value_t = 60)]
    fps: u32,
    #[arg(long, default_value_t = 20_000)]
    bitrate_kbps: u32,
    #[arg(long, default_value = "av1")]
    codec: String,
    #[arg(long)]
    encoder: Option<String>,
    #[arg(long, default_value_t = 8.0)]
    mouse_smoothing_ms: f32,
    #[arg(long, default_value = "pairing-key")]
    pairing_key_file: PathBuf,
    #[arg(last = true, required = true)]
    command: Vec<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    start_audio_sink()?;
    let mut robot = open_robot(&args)?;
    let (width, height) = wait_for_first_display(&robot, args.max_width, args.max_height)?;
    let previous_x_displays = x_displays()?;
    start_gamescope(&args, width, height)?;
    let input_socket =
        PathBuf::from(std::env::var("XDG_RUNTIME_DIR")?).join(GAMESCOPE_INPUT_SOCKET);
    wait_until(|| Ok(input_socket.exists()))?;
    let x_display = wait_for_new_x_display(&previous_x_displays)?;

    let track = robot.video_with_options("screen", &video_options(&args, width, height))?;
    let audio_source = format!(
        "pipewiresrc target-object={AUDIO_SINK} stream-properties={MONITOR_CAPTURE} ! audio/x-raw,channels=2"
    );
    let audio_options = AudioOptions {
        channels: AudioChannels::Stereo,
        bitrate_kbps: 128,
        ..AudioOptions::default()
    };
    robot.attach_audio_gst("audio", &audio_source, &audio_options)?;

    let robot = Arc::new(robot);
    let emulator = emulator::Emulator::connect(&input_socket)?;
    keyboard::forward_keyboard(&robot, emulator.clone())?;
    mouse::forward_mouse(&robot, emulator, args.mouse_smoothing_ms)?;
    gamepad::forward_gamepads(&robot)?;
    let cursor = cursor::CursorReader::connect(&x_display, width, height)?;
    capture::stream_gamescope(track, cursor, width, height, args.fps)?;
    robot.run()?;
    Ok(())
}

fn start_audio_sink() -> Result<(), Box<dyn Error>> {
    let modules = Command::new("pactl")
        .args(["list", "short", "modules"])
        .output()?
        .stdout;
    if !String::from_utf8(modules)?.contains(&format!("sink_name={AUDIO_SINK} ")) {
        Command::new("pactl")
            .args([
                "load-module",
                "module-null-sink",
                &format!("sink_name={AUDIO_SINK}"),
                "sink_properties=device.description=Cloud-Gaming",
            ])
            .status()?;
    }
    Ok(())
}

fn wait_for_first_display(
    robot: &Robot,
    max_width: u32,
    max_height: u32,
) -> Result<(u32, u32), Box<dyn Error>> {
    let (sender, receiver) = mpsc::channel();
    robot.on_control("json/display", move |sample| {
        if let ControlMessage::Raw(display) = sample.message {
            let _ = sender.send(display);
        }
    })?;
    let display = receiver.recv()?;
    let (width, height) = (display["width"].as_f64().unwrap(), display["height"].as_f64().unwrap());
    let scale = (f64::from(max_width) / width)
        .min(f64::from(max_height) / height)
        .min(1.0);
    let even = |size: f64| (size * scale) as u32 & !1;
    Ok((even(width), even(height)))
}

fn start_gamescope(args: &Args, width: u32, height: u32) -> Result<(), Box<dyn Error>> {
    let (width, height, fps) = (
        width.to_string(),
        height.to_string(),
        args.fps.to_string(),
    );
    Command::new("systemd-inhibit")
        .args([
            "--what=sleep:idle",
            "--who=Adamo cloud gaming",
            "--why=Streaming a game",
        ])
        .args([
            "gamescope",
            "-e",
            "--backend",
            "headless",
            "-W",
            &width,
            "-H",
            &height,
            "-w",
            &width,
            "-h",
            &height,
            "-r",
            &fps,
            "--",
        ])
        .args(&args.command)
        .env("PULSE_SINK", AUDIO_SINK)
        .spawn()?;
    Ok(())
}

fn open_robot(args: &Args) -> Result<Robot, Box<dyn Error>> {
    let saved_pairing_key = std::fs::read_to_string(&args.pairing_key_file)
        .ok()
        .map(|key| key.trim().to_owned());
    let options = P2pOptions {
        pairing_key: saved_pairing_key,
        ..P2pOptions::default()
    };
    let robot = Robot::new_local_with_options(&args.name, options)?;
    let pairing_key = robot
        .session()
        .local_pairing_key()
        .ok_or("the local gateway did not start")?;
    let port = robot
        .session()
        .local_port()
        .ok_or("the local gateway did not start")?;
    save_secret(&args.pairing_key_file, &pairing_key)?;
    println!(
        "Connect from the player to this PC's address, port {port}, with pairing key {pairing_key}"
    );
    Ok(robot)
}

fn video_options(args: &Args, width: u32, height: u32) -> VideoOptions {
    VideoOptions {
        width,
        height,
        fps: args.fps,
        codec: args.codec.clone(),
        encoder: args.encoder.clone(),
        bitrate_kbps: args.bitrate_kbps,
        rate_control: Some("cbr".into()),
        intra_refresh: false,
        keyframe_distance: 3600.0,
        ..VideoOptions::default().with_pixel_format("BGRX")
    }
}

fn save_secret(path: &Path, secret: &str) -> std::io::Result<()> {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)?;
    file.write_all(secret.as_bytes())
}

fn wait_for_new_x_display(previous_x_displays: &HashSet<String>) -> Result<String, Box<dyn Error>> {
    loop {
        if let Some(display) = x_displays()?.difference(previous_x_displays).next() {
            return Ok(display.clone());
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn wait_until(is_ready: impl Fn() -> std::io::Result<bool>) -> std::io::Result<()> {
    while !is_ready()? {
        std::thread::sleep(Duration::from_millis(100));
    }
    Ok(())
}

fn x_displays() -> std::io::Result<HashSet<String>> {
    let sockets = std::fs::read_dir("/tmp/.X11-unix")?.flatten();
    Ok(sockets
        .filter_map(|entry| {
            Some(format!(
                ":{}",
                entry.file_name().to_str()?.strip_prefix('X')?
            ))
        })
        .collect())
}
