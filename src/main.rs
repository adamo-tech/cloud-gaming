mod capture;
mod cursor;
mod emulator;
mod gamepad;
mod keyboard;
mod mouse;

use std::error::Error;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, RwLock};
use std::time::Duration;

use adamo::p2p::P2pOptions;
use adamo::{
    AudioChannels, AudioOptions, ControlMessage, DirectOptions, Protocol, Robot, VideoOptions,
};
use clap::Parser;

const AUDIO_SINK: &str = "cloud_gaming_audio";
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
    #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
    experimental: bool,
    #[arg(long, default_value = "pairing-key")]
    pairing_key_file: PathBuf,
    #[arg(last = true, required = true)]
    command: Vec<String>,
}

struct Gamescope {
    process: Child,
    input_socket: PathBuf,
    x_display: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    start_audio_sink()?;
    let mut robot = open_robot(&args)?;
    robot.set_experimental(args.experimental)?;
    let audio_source = format!("pulsesrc device={AUDIO_SINK}.monitor ! audio/x-raw,channels=2");
    let audio_options = AudioOptions {
        channels: AudioChannels::Stereo,
        bitrate_kbps: 128,
        ..AudioOptions::default()
    };
    robot.attach_audio_gst("audio", &audio_source, &audio_options)?;
    let displays = watch_displays(&robot, args.max_width, args.max_height)?;
    let emulator = emulator::Emulator::new();
    keyboard::forward_keyboard(&robot, emulator.clone())?;
    mouse::forward_mouse(&robot, emulator.clone(), args.mouse_smoothing_ms)?;
    let robot = Arc::new(RwLock::new(robot));
    gamepad::forward_gamepads(&robot)?;

    let mut size = displays.recv()?;
    loop {
        let gamescope = Gamescope::start(&args, size)?;
        emulator.connect(&gamescope.input_socket)?;
        let track = {
            let mut robot = robot.write().unwrap();
            robot.remove_video("screen");
            robot.video_with_options("screen", &video_options(&args, size))?
        };
        let cursor = cursor::CursorReader::connect(&gamescope.x_display, size.0, size.1)?;
        capture::stream_gamescope(track, cursor, size.0, size.1, args.fps)?;
        size = run_until_resized(&robot.read().unwrap(), &displays, size)?;
        gamescope.stop()?;
    }
}

fn run_until_resized(
    robot: &Robot,
    displays: &Receiver<(u32, u32)>,
    size: (u32, u32),
) -> Result<(u32, u32), Box<dyn Error>> {
    std::thread::scope(|scope| -> Result<(u32, u32), Box<dyn Error>> {
        let run = scope.spawn(|| robot.run());
        let new_size = loop {
            let display = displays.recv()?;
            if display != size {
                break display;
            }
        };
        robot.stop()?;
        run.join().unwrap()?;
        Ok(new_size)
    })
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
    Command::new("pactl")
        .args(["set-default-sink", AUDIO_SINK])
        .status()?;
    Ok(())
}

fn watch_displays(
    robot: &Robot,
    max_width: u32,
    max_height: u32,
) -> adamo::Result<Receiver<(u32, u32)>> {
    let (sender, receiver) = mpsc::channel();
    robot.on_control("json/display", move |sample| {
        if let ControlMessage::Raw(display) = sample.message {
            let (width, height) = (
                display["width"].as_f64().unwrap(),
                display["height"].as_f64().unwrap(),
            );
            let scale = (f64::from(max_width) / width)
                .min(f64::from(max_height) / height)
                .min(1.0);
            let even = |size: f64| (size * scale) as u32 & !1;
            let _ = sender.send((even(width), even(height)));
        }
    })?;
    Ok(receiver)
}

impl Gamescope {
    fn start(args: &Args, (width, height): (u32, u32)) -> Result<Self, Box<dyn Error>> {
        let (width, height, fps) = (width.to_string(), height.to_string(), args.fps.to_string());
        let is_steam = Path::new(&args.command[0]).ends_with("steam");
        let process = Command::new("systemd-inhibit")
            .args([
                "--what=sleep:idle",
                "--who=Adamo cloud gaming",
                "--why=Streaming a game",
            ])
            .arg("gamescope")
            .args(is_steam.then_some("-e"))
            .args([
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
            .process_group(0)
            .spawn()?;
        let input_socket =
            PathBuf::from(std::env::var("XDG_RUNTIME_DIR")?).join(GAMESCOPE_INPUT_SOCKET);
        wait_for(|| Ok(input_socket.exists().then_some(())))?;
        let x_display = wait_for(|| x_display_of(process.id() as libc::pid_t))?;
        wait_for(|| Ok(has_gamescope_video_node()?.then_some(())))?;
        Ok(Self {
            process,
            input_socket,
            x_display,
        })
    }

    fn stop(mut self) -> std::io::Result<()> {
        let process_group = self.process.id() as libc::pid_t;
        unsafe { libc::killpg(process_group, libc::SIGTERM) };
        self.process.wait()?;
        wait_for(|| Ok((unsafe { libc::killpg(process_group, 0) } != 0).then_some(())))
    }
}

fn open_robot(args: &Args) -> Result<Robot, Box<dyn Error>> {
    let saved_pairing_key = std::fs::read_to_string(&args.pairing_key_file)
        .ok()
        .map(|key| key.trim().to_owned());
    let api_key = std::env::var("ADAMO_API_KEY").ok();
    let robot = match &api_key {
        Some(api_key) => {
            let options = DirectOptions {
                pairing_key: saved_pairing_key,
                ..DirectOptions::default()
            };
            Robot::new_with_direct_options(
                Some(api_key),
                Some(&args.name),
                Protocol::default(),
                options,
            )?
        }
        None => {
            let options = P2pOptions {
                pairing_key: saved_pairing_key,
                ..P2pOptions::default()
            };
            Robot::new_local_with_options(&args.name, options)?
        }
    };
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
    if api_key.is_some() {
        println!(
            "Or connect through the Adamo relay to {} with an API key of your organization",
            args.name
        );
    }
    Ok(robot)
}

fn video_options(args: &Args, (width, height): (u32, u32)) -> VideoOptions {
    VideoOptions {
        width,
        height,
        fps: args.fps,
        codec: args.codec.clone(),
        encoder: args.encoder.clone(),
        bitrate_kbps: args.bitrate_kbps,
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

fn wait_for<T>(ready: impl Fn() -> std::io::Result<Option<T>>) -> std::io::Result<T> {
    loop {
        if let Some(value) = ready()? {
            return Ok(value);
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn has_gamescope_video_node() -> std::io::Result<bool> {
    let nodes = Command::new("pw-cli").args(["ls", "Node"]).output()?;
    Ok(String::from_utf8_lossy(&nodes.stdout).contains("node.name = \"gamescope\""))
}

fn x_display_of(process_group: libc::pid_t) -> std::io::Result<Option<String>> {
    let sockets = std::fs::read_dir("/tmp/.X11-unix")?.flatten();
    Ok(sockets
        .filter_map(|entry| {
            let number = entry.file_name().to_str()?.strip_prefix('X')?.to_owned();
            let lock = std::fs::read_to_string(format!("/tmp/.X{number}-lock")).ok()?;
            let owner = lock
                .trim_matches(|c: char| c.is_whitespace() || c == '\0')
                .parse()
                .ok()?;
            (unsafe { libc::getpgid(owner) } == process_group).then(|| format!(":{number}"))
        })
        .next())
}
