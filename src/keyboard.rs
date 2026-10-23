use std::collections::HashSet;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use adamo::{ControlMessage, Robot};
use keycode::{KeyMap, KeyMapping, KeyMappingCode};

use crate::emulator::Emulator;

pub fn forward_keyboard(robot: &Robot, emulator: Arc<Emulator>) -> adamo::Result<()> {
    let pressed_keys = Mutex::new(HashSet::new());
    robot.on_control("json/keyboard", move |sample| {
        let ControlMessage::Keyboard(keyboard) = sample.message else {
            return;
        };
        let Some(key) = evdev_key(&keyboard.code) else {
            return;
        };
        let mut pressed_keys = pressed_keys.lock().unwrap();
        let is_changed = if keyboard.is_down() {
            pressed_keys.insert(key)
        } else {
            pressed_keys.remove(&key)
        };
        if is_changed {
            emulator.press_key(key, keyboard.is_down());
        }
    })
}

fn evdev_key(code: &str) -> Option<u32> {
    let code = KeyMappingCode::from_str(code).ok()?;
    let key = KeyMap::from_key_mapping(KeyMapping::Code(Some(code))).ok()?;
    Some(u32::from(key.evdev))
}
