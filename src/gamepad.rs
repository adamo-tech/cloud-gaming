use std::collections::HashMap;
use std::sync::mpsc::{self, RecvTimeoutError};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

use adamo::{ControlMessage, Joy, Robot, Rumble};
use inputtino::{DeviceDefinition, JoypadButton, JoypadStickPosition, XboxOneJoypad};

const STANDARD_BUTTONS: [(usize, JoypadButton); 15] = [
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
const RUMBLE_REFRESH: Duration = Duration::from_millis(200);

type ActiveRumbles = Arc<Mutex<HashMap<u8, Rumble>>>;

pub fn forward_gamepads(robot: &Arc<RwLock<Robot>>) -> adamo::Result<()> {
    let (joy_sender, joy_receiver) = mpsc::channel::<(u8, Joy)>();
    let thread_robot = robot.clone();
    std::thread::spawn(move || {
        let active_rumbles = ActiveRumbles::default();
        let mut gamepads = HashMap::from([(0, create_gamepad(0, &thread_robot, &active_rumbles))]);
        let mut last_refresh = Instant::now();
        loop {
            match joy_receiver.recv_timeout(RUMBLE_REFRESH) {
                Ok((index, joy)) => {
                    let gamepad = gamepads
                        .entry(index)
                        .or_insert_with(|| create_gamepad(index, &thread_robot, &active_rumbles));
                    apply_joy(gamepad, &joy);
                }
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => break,
            }
            if last_refresh.elapsed() >= RUMBLE_REFRESH {
                for rumble in active_rumbles.lock().unwrap().values() {
                    let _ = thread_robot.read().unwrap().send_rumble(rumble);
                }
                last_refresh = Instant::now();
            }
        }
    });
    robot.read().unwrap().on_control("joy/**", move |sample| {
        let joy = match sample.message {
            ControlMessage::Joy(joy) => joy,
            ControlMessage::JoystickCommand(command) => Joy {
                axes: command.axes,
                buttons: command.buttons,
                stamp: command.stamp,
            },
            _ => return,
        };
        let index = sample
            .channel
            .strip_prefix("joy/")
            .and_then(|index| index.parse().ok())
            .unwrap_or(0);
        let _ = joy_sender.send((index, joy));
    })
}

fn create_gamepad(
    index: u8,
    robot: &Arc<RwLock<Robot>>,
    active_rumbles: &ActiveRumbles,
) -> XboxOneJoypad {
    let definition =
        DeviceDefinition::new("Adamo virtual Xbox One pad", 0x045E, 0x02EA, 0x0408, "", "");
    let mut gamepad = XboxOneJoypad::new(&definition)
        .expect("creating a virtual gamepad needs write access to /dev/uinput");
    let robot = robot.clone();
    let active_rumbles = active_rumbles.clone();
    gamepad.set_on_rumble(move |strong, weak| {
        let rumble = Rumble {
            gamepad_index: index,
            strong_magnitude: magnitude(strong),
            weak_magnitude: magnitude(weak),
            duration_ms: RUMBLE_DURATION_MS,
        };
        let _ = robot.read().unwrap().send_rumble(&rumble);
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
    let is_pressed = |index: usize| joy.buttons.get(index).is_some_and(|value| *value != 0);
    let pressed_buttons = STANDARD_BUTTONS
        .iter()
        .filter(|(index, _)| is_pressed(*index))
        .fold(0, |flags, (_, button)| flags | *button as i32);
    gamepad.set_pressed(pressed_buttons);
    gamepad.set_stick(JoypadStickPosition::LS, stick(axis(0)), stick(-axis(1)));
    gamepad.set_stick(JoypadStickPosition::RS, stick(axis(2)), stick(-axis(3)));
    gamepad.set_triggers(trigger(axis(4)), trigger(axis(5)));
}

fn magnitude(level: i32) -> f32 {
    (level.unsigned_abs() as f32 / 65535.0).min(1.0)
}

fn stick(value: f64) -> i16 {
    (value * 32767.0) as i16
}

fn trigger(value: f64) -> i16 {
    (value * 255.0) as i16
}
