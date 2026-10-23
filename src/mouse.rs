use std::sync::{Arc, Mutex};
use std::time::Duration;

use adamo::{ControlMessage, Mouse, Robot};

use crate::emulator::Emulator;

const BTN_LEFT: u32 = 0x110;
const BTN_RIGHT: u32 = 0x111;
const BTN_MIDDLE: u32 = 0x112;
const BTN_SIDE: u32 = 0x113;
const BTN_EXTRA: u32 = 0x114;
const BROWSER_BUTTONS: [(u8, u32); 5] = [
    (1, BTN_LEFT),
    (2, BTN_RIGHT),
    (4, BTN_MIDDLE),
    (8, BTN_SIDE),
    (16, BTN_EXTRA),
];
const SMOOTHING_STEP: Duration = Duration::from_millis(1);
const SMOOTHING_REST: f32 = 0.05;

pub fn forward_mouse(
    robot: &Robot,
    emulator: Arc<Emulator>,
    smoothing_ms: f32,
) -> adamo::Result<()> {
    let pending_motion = Arc::new(Mutex::new((0.0f32, 0.0f32)));
    if smoothing_ms > 0.0 {
        spawn_smoothing(emulator.clone(), pending_motion.clone(), smoothing_ms);
    }
    let last_mouse = Mutex::new(None::<Mouse>);
    robot.on_control("json/mouse", move |sample| {
        let ControlMessage::Mouse(mouse) = sample.message else {
            return;
        };
        let mut last_mouse = last_mouse.lock().unwrap();
        let previous = last_mouse.clone().unwrap_or_else(|| Mouse {
            button_mask: 0,
            ..mouse.clone()
        });
        if mouse.stamp < previous.stamp {
            return;
        }

        let motion = (
            (mouse.motion_x_total - previous.motion_x_total) as f32,
            (mouse.motion_y_total - previous.motion_y_total) as f32,
        );
        if motion != (0.0, 0.0) && smoothing_ms > 0.0 {
            let mut pending = pending_motion.lock().unwrap();
            *pending = (pending.0 + motion.0, pending.1 + motion.1);
        } else if motion != (0.0, 0.0) {
            emulator.move_pointer(motion.0, motion.1);
        }

        for (bit, button) in BROWSER_BUTTONS {
            if (mouse.button_mask ^ previous.button_mask) & bit != 0 {
                emulator.press_button(button, mouse.button_mask & bit != 0);
            }
        }

        let wheel = (
            mouse.horizontal_wheel_total - previous.horizontal_wheel_total,
            mouse.wheel_total - previous.wheel_total,
        );
        if wheel != (0, 0) {
            emulator.scroll(wheel.0 as i32, wheel.1 as i32);
        }
        *last_mouse = Some(mouse);
    })
}

fn spawn_smoothing(
    emulator: Arc<Emulator>,
    pending_motion: Arc<Mutex<(f32, f32)>>,
    smoothing_ms: f32,
) {
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(SMOOTHING_STEP);
            let step = {
                let mut pending = pending_motion.lock().unwrap();
                let is_at_rest = pending.0.abs() + pending.1.abs() < SMOOTHING_REST;
                let step = if is_at_rest {
                    *pending
                } else {
                    (pending.0 / smoothing_ms, pending.1 / smoothing_ms)
                };
                *pending = (pending.0 - step.0, pending.1 - step.1);
                step
            };
            if step != (0.0, 0.0) {
                emulator.move_pointer(step.0, step.1);
            }
        }
    });
}
