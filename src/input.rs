use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::str::FromStr;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use adamo::{ControlMessage, Joy, Mouse, Robot, Rumble};
use inputtino::{DeviceDefinition, JoypadButton, JoypadStickPosition, XboxOneJoypad};
use keycode::{KeyMap, KeyMapping, KeyMappingCode};
use reis::ei;
use reis::event::{Device, DeviceCapability, EiEvent};

const MOUSE_BUTTONS: [(u8, u32); 5] = [(1, 0x110), (2, 0x111), (4, 0x112), (8, 0x113), (16, 0x114)];
const GAMEPAD_BUTTONS: [(usize, JoypadButton); 15] = [
    (0, JoypadButton::A),
    (1, JoypadButton::B),
    (2, JoypadButton::X),
    (3, JoypadButton::Y),
    (4, JoypadButton::LEFT_BUTTON),
    (5, JoypadButton::RIGHT_BUTTON),
    (8, JoypadButton::BACK),
    (9, JoypadButton::START),
    (10, JoypadButton::LEFT_STICK),
    (11, JoypadButton::RIGHT_STICK),
    (12, JoypadButton::DPAD_UP),
    (13, JoypadButton::DPAD_DOWN),
    (14, JoypadButton::DPAD_LEFT),
    (15, JoypadButton::DPAD_RIGHT),
    (16, JoypadButton::HOME),
];
const RUMBLE_DURATION_MS: u32 = 300;
const MOTION_STEP_INTERVAL: Duration = Duration::from_millis(1);
const MOTION_STEP_FRACTION: f32 = 1.0 / 8.0;
const MOTION_REST: f32 = 0.05;
const RUMBLE_REFRESH: Duration = Duration::from_millis(200);

pub struct Emulator {
    connection: reis::event::Connection,
    devices: Mutex<Vec<Device>>,
    pending_motion: Mutex<(f32, f32)>,
    sequence: AtomicU32,
    started: Instant,
}

impl Emulator {
    pub fn connect(socket: &Path) -> Result<Arc<Self>, Box<dyn Error>> {
        let stream = UnixStream::connect(socket)?;
        let (emulator_sender, emulator_receiver) = mpsc::channel();
        std::thread::spawn(move || {
            let context = ei::Context::new(stream).unwrap();
            let (connection, events) = context.handshake_blocking("Adamo cloud gaming", ei::handshake::ContextType::Sender).unwrap();
            let emulator = Arc::new(Self { connection, devices: Mutex::new(Vec::new()), pending_motion: Mutex::new((0.0, 0.0)), sequence: AtomicU32::new(0), started: Instant::now() });
            emulator_sender.send(emulator.clone()).unwrap();
            for event in events.flatten() {
                match event {
                    EiEvent::SeatAdded(added) => added.seat.bind_capabilities(DeviceCapability::Pointer | DeviceCapability::Keyboard | DeviceCapability::Button | DeviceCapability::Scroll),
                    EiEvent::DeviceResumed(resumed) => {
                        resumed.device.device().start_emulating(resumed.serial, emulator.sequence.fetch_add(1, Ordering::Relaxed));
                        emulator.devices.lock().unwrap().push(resumed.device);
                    }
                    EiEvent::DevicePaused(paused) => emulator.devices.lock().unwrap().retain(|device| *device != paused.device),
                    EiEvent::DeviceRemoved(removed) => emulator.devices.lock().unwrap().retain(|device| *device != removed.device),
                    _ => {}
                }
                let _ = emulator.connection.flush();
            }
        });
        let emulator: Arc<Self> = emulator_receiver.recv()?;
        let stepping_emulator = emulator.clone();
        std::thread::spawn(move || loop {
            std::thread::sleep(MOTION_STEP_INTERVAL);
            let step = {
                let mut pending = stepping_emulator.pending_motion.lock().unwrap();
                let step = if pending.0.abs() + pending.1.abs() < MOTION_REST { *pending } else { (pending.0 * MOTION_STEP_FRACTION, pending.1 * MOTION_STEP_FRACTION) };
                pending.0 -= step.0;
                pending.1 -= step.1;
                step
            };
            if step != (0.0, 0.0) {
                stepping_emulator.emit(DeviceCapability::Pointer, |device| device.interface::<ei::Pointer>().unwrap().motion_relative(step.0, step.1));
            }
        });
        Ok(emulator)
    }

    fn emit(&self, capability: DeviceCapability, request: impl Fn(&Device)) {
        let devices = self.devices.lock().unwrap();
        let Some(device) = devices.iter().find(|device| device.has_capability(capability)) else { return };
        request(device);
        device.device().frame(self.connection.serial(), self.started.elapsed().as_micros() as u64);
        let _ = self.connection.flush();
    }
}

pub fn forward_keyboard(robot: &Robot, emulator: Arc<Emulator>) -> adamo::Result<()> {
    let pressed_keys = Mutex::new(HashSet::new());
    robot.on_control("json/keyboard", move |sample| {
        let ControlMessage::Keyboard(keyboard) = sample.message else { return };
        let Some(key) = evdev_key(&keyboard.code) else { return };
        let is_changed = if keyboard.is_down() { pressed_keys.lock().unwrap().insert(key) } else { pressed_keys.lock().unwrap().remove(&key) };
        if is_changed {
            let state = if keyboard.is_down() { ei::keyboard::KeyState::Press } else { ei::keyboard::KeyState::Released };
            emulator.emit(DeviceCapability::Keyboard, |device| device.interface::<ei::Keyboard>().unwrap().key(key, state));
        }
    })
}

pub fn forward_mouse(robot: &Robot, emulator: Arc<Emulator>) -> adamo::Result<()> {
    let last_mouse = Mutex::new(None::<Mouse>);
    robot.on_control("json/mouse", move |sample| {
        let ControlMessage::Mouse(mouse) = sample.message else { return };
        let mut last_mouse = last_mouse.lock().unwrap();
        let previous = last_mouse.clone().unwrap_or_else(|| Mouse { button_mask: 0, ..mouse.clone() });
        if mouse.stamp < previous.stamp {
            return;
        }
        let motion = (mouse.motion_x_total - previous.motion_x_total, mouse.motion_y_total - previous.motion_y_total);
        if motion != (0, 0) {
            let mut pending = emulator.pending_motion.lock().unwrap();
            pending.0 += motion.0 as f32;
            pending.1 += motion.1 as f32;
        }
        for (bit, button) in MOUSE_BUTTONS {
            if (mouse.button_mask ^ previous.button_mask) & bit != 0 {
                let state = if mouse.button_mask & bit != 0 { ei::button::ButtonState::Press } else { ei::button::ButtonState::Released };
                emulator.emit(DeviceCapability::Button, |device| device.interface::<ei::Button>().unwrap().button(button, state));
            }
        }
        let wheel = (mouse.horizontal_wheel_total - previous.horizontal_wheel_total, mouse.wheel_total - previous.wheel_total);
        if wheel != (0, 0) {
            emulator.emit(DeviceCapability::Scroll, |device| device.interface::<ei::Scroll>().unwrap().scroll_discrete(wheel.0 as i32, wheel.1 as i32));
        }
        *last_mouse = Some(mouse);
    })
}

pub fn forward_gamepads(robot: &Arc<Robot>) -> adamo::Result<()> {
    let (joy_sender, joy_receiver) = mpsc::channel::<(u8, Joy)>();
    let thread_robot = robot.clone();
    std::thread::spawn(move || {
        let active_rumbles = Arc::new(Mutex::new(HashMap::new()));
        let mut gamepads = HashMap::new();
        let mut last_refresh = Instant::now();
        loop {
            match joy_receiver.recv_timeout(RUMBLE_REFRESH) {
                Ok((index, joy)) => apply_joy(gamepads.entry(index).or_insert_with(|| create_gamepad(index, &thread_robot, &active_rumbles)), &joy),
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => break,
            }
            if last_refresh.elapsed() >= RUMBLE_REFRESH {
                for rumble in active_rumbles.lock().unwrap().values() {
                    let _ = thread_robot.send_rumble(rumble);
                }
                last_refresh = Instant::now();
            }
        }
    });
    robot.on_control("joy/**", move |sample| {
        let joy = match sample.message {
            ControlMessage::Joy(joy) => joy,
            ControlMessage::JoystickCommand(command) => Joy { axes: command.axes, buttons: command.buttons, stamp: command.stamp },
            _ => return,
        };
        let index = sample.channel.strip_prefix("joy/").and_then(|index| index.parse().ok()).unwrap_or(0);
        let _ = joy_sender.send((index, joy));
    })
}

fn evdev_key(code: &str) -> Option<u32> {
    let code = KeyMappingCode::from_str(code).ok()?;
    KeyMap::from_key_mapping(KeyMapping::Code(Some(code))).ok().map(|key| u32::from(key.evdev))
}

fn create_gamepad(index: u8, robot: &Arc<Robot>, active_rumbles: &Arc<Mutex<HashMap<u8, Rumble>>>) -> XboxOneJoypad {
    let mut gamepad = XboxOneJoypad::new(&DeviceDefinition::new("Adamo virtual Xbox One pad", 0x045E, 0x02EA, 0x0408, "", "")).expect("creating a virtual gamepad needs write access to /dev/uinput");
    let robot = robot.clone();
    let active_rumbles = active_rumbles.clone();
    gamepad.set_on_rumble(move |strong, weak| {
        let rumble = Rumble { gamepad_index: index, strong_magnitude: (strong as f32 / 65535.0).min(1.0), weak_magnitude: (weak as f32 / 65535.0).min(1.0), duration_ms: RUMBLE_DURATION_MS };
        let _ = robot.send_rumble(&rumble);
        let mut active_rumbles = active_rumbles.lock().unwrap();
        if strong == 0 && weak == 0 {
            active_rumbles.remove(&index);
        } else {
            active_rumbles.insert(index, rumble);
        }
    });
    gamepad
}

fn apply_joy(gamepad: &XboxOneJoypad, joy: &Joy) {
    let axis = |index: usize| joy.axes.get(index).copied().unwrap_or(0.0);
    let pressed_buttons = GAMEPAD_BUTTONS.iter().filter(|(index, _)| joy.buttons.get(*index).is_some_and(|value| *value != 0)).fold(0, |flags, (_, button)| flags | *button as i32);
    gamepad.set_pressed(pressed_buttons);
    gamepad.set_stick(JoypadStickPosition::LS, stick(axis(0)), stick(-axis(1)));
    gamepad.set_stick(JoypadStickPosition::RS, stick(axis(2)), stick(-axis(3)));
    gamepad.set_triggers(trigger(axis(4)), trigger(axis(5)));
}

fn stick(value: f64) -> i16 {
    (value * 32767.0) as i16
}

fn trigger(value: f64) -> i16 {
    (value * 255.0) as i16
}
