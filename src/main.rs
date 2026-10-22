mod cursor;
mod input;

use std::collections::HashSet;
use std::error::Error;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;

use adamo::p2p::P2pOptions;
use adamo::{AudioChannels, AudioOptions, Robot, VideoOptions};
use clap::Parser;

const AUDIO_SINK: &str = "cloud_gaming_audio";
const GAMESCOPE_INPUT_SOCKET: &str = "gamescope-0-ei";

#[derive(Parser)]
struct Args {
    #[arg(long, default_value = "cloud-gaming")]
    name: String,
    #[arg(long, default_value_t = 1920)]
    width: u32,
    #[arg(long, default_value_t = 1080)]
    height: u32,
    #[arg(long, default_value_t = 60)]
    fps: u32,
    #[arg(long, default_value_t = 20_000)]
    bitrate_kbps: u32,
    #[arg(long, default_value = "pairing-key")]
    pairing_key_file: PathBuf,
    #[arg(last = true, required = true)]
    command: Vec<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let modules = String::from_utf8(Command::new("pactl").args(["list", "short", "modules"]).output()?.stdout)?;
    if !modules.contains(&format!("sink_name={AUDIO_SINK} ")) {
        Command::new("pactl").args(["load-module", "module-null-sink", &format!("sink_name={AUDIO_SINK}"), "sink_properties=device.description=Cloud-Gaming"]).status()?;
    }

    let previous_x_displays = x_displays()?;
    Command::new("systemd-inhibit")
        .args(["--what=sleep:idle", "--who=Adamo cloud gaming", "--why=Streaming a game", "gamescope", "--backend", "headless", "--force-grab-cursor", "-W", &args.width.to_string(), "-H", &args.height.to_string(), "-r", &args.fps.to_string(), "--"])
        .args(&args.command)
        .env("PULSE_SINK", AUDIO_SINK)
        .spawn()?;
    let input_socket = PathBuf::from(std::env::var("XDG_RUNTIME_DIR")?).join(GAMESCOPE_INPUT_SOCKET);
    while !input_socket.exists() {
        std::thread::sleep(Duration::from_millis(100));
    }
    let x_display = loop {
        if let Some(display) = x_displays()?.difference(&previous_x_displays).next() {
            break display.clone();
        }
        std::thread::sleep(Duration::from_millis(100));
    };

    let saved_pairing_key = std::fs::read_to_string(&args.pairing_key_file).ok().map(|key| key.trim().to_owned());
    let mut robot = Robot::new_local_with_options(&args.name, P2pOptions { pairing_key: saved_pairing_key, ..P2pOptions::default() })?;
    let pairing_key = robot.session().local_pairing_key().ok_or("the local gateway did not start")?;
    std::fs::OpenOptions::new().write(true).create(true).truncate(true).mode(0o600).open(&args.pairing_key_file)?.write_all(pairing_key.as_bytes())?;
    println!("Connect from the player to this PC's address, port {}, with pairing key {pairing_key}", robot.session().local_port().ok_or("the local gateway did not start")?);
    robot.attach_gst_with_options("screen", "pipewiresrc target-object=gamescope always-copy=true keepalive-time=100 do-timestamp=true ! capsfilter caps=video/x-raw,format=BGRx", &VideoOptions {
        width: args.width,
        height: args.height,
        fps: args.fps,
        codec: "av1".into(),
        bitrate_kbps: args.bitrate_kbps,
        rate_control: Some("cbr".into()),
        intra_refresh: false,
        keyframe_distance: 3600.0,
        ..VideoOptions::default()
    })?;
    robot.attach_audio_gst("audio", &format!("pipewiresrc target-object={AUDIO_SINK} stream-properties=\"props,stream.capture.sink=true\""), &AudioOptions { channels: AudioChannels::Stereo, bitrate_kbps: 128, ..AudioOptions::default() })?;

    let robot = Arc::new(robot);
    let emulator = input::Emulator::connect(&input_socket)?;
    input::forward_keyboard(&robot, emulator.clone())?;
    input::forward_mouse(&robot, emulator)?;
    input::forward_gamepads(&robot)?;
    cursor::publish_cursor(robot.clone(), &x_display, args.width, args.height)?;
    robot.run()?;
    Ok(())
}

fn x_displays() -> std::io::Result<HashSet<String>> {
    Ok(std::fs::read_dir("/tmp/.X11-unix")?.flatten().filter_map(|entry| entry.file_name().to_str()?.strip_prefix('X').map(|number| format!(":{number}"))).collect())
}
