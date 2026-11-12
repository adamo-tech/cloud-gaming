use std::error::Error;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use reis::ei;
use reis::event::{Connection, Device, DeviceCapability, EiEvent};

pub struct Emulator {
    connection: Mutex<Option<Connection>>,
    devices: Mutex<Vec<Device>>,
    sequence: AtomicU32,
    started: Instant,
}

impl Emulator {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            connection: Mutex::new(None),
            devices: Mutex::new(Vec::new()),
            sequence: AtomicU32::new(0),
            started: Instant::now(),
        })
    }

    pub fn connect(self: &Arc<Self>, socket: &Path) -> Result<(), Box<dyn Error>> {
        let stream = UnixStream::connect(socket)?;
        let (connected_sender, connected_receiver) = mpsc::channel();
        let emulator = self.clone();
        std::thread::spawn(move || {
            let context = ei::Context::new(stream).unwrap();
            let (connection, events) = context
                .handshake_blocking("Adamo cloud gaming", ei::handshake::ContextType::Sender)
                .unwrap();
            emulator.devices.lock().unwrap().clear();
            *emulator.connection.lock().unwrap() = Some(connection);
            connected_sender.send(()).unwrap();
            for event in events.flatten() {
                emulator.handle(event);
            }
        });
        Ok(connected_receiver.recv()?)
    }

    pub fn press_key(&self, key: u32, is_pressed: bool) {
        let state = if is_pressed {
            ei::keyboard::KeyState::Press
        } else {
            ei::keyboard::KeyState::Released
        };
        self.emit::<ei::Keyboard>(DeviceCapability::Keyboard, |keyboard| {
            keyboard.key(key, state)
        });
    }

    pub fn move_pointer(&self, x: f32, y: f32) {
        self.emit::<ei::Pointer>(DeviceCapability::Pointer, |pointer| {
            pointer.motion_relative(x, y)
        });
    }

    pub fn press_button(&self, button: u32, is_pressed: bool) {
        let state = if is_pressed {
            ei::button::ButtonState::Press
        } else {
            ei::button::ButtonState::Released
        };
        self.emit::<ei::Button>(DeviceCapability::Button, |buttons| {
            buttons.button(button, state)
        });
    }

    pub fn scroll(&self, x_v120: i32, y_v120: i32) {
        self.emit::<ei::Scroll>(DeviceCapability::Scroll, |scroll| {
            scroll.scroll_discrete(x_v120, y_v120)
        });
    }

    fn handle(&self, event: EiEvent) {
        match event {
            EiEvent::SeatAdded(added) => {
                let capabilities = DeviceCapability::Pointer
                    | DeviceCapability::Keyboard
                    | DeviceCapability::Button
                    | DeviceCapability::Scroll;
                added.seat.bind_capabilities(capabilities);
            }
            EiEvent::DeviceResumed(resumed) => {
                resumed.device.device().start_emulating(
                    resumed.serial,
                    self.sequence.fetch_add(1, Ordering::Relaxed),
                );
                self.devices.lock().unwrap().push(resumed.device);
            }
            EiEvent::DevicePaused(paused) => self
                .devices
                .lock()
                .unwrap()
                .retain(|device| *device != paused.device),
            EiEvent::DeviceRemoved(removed) => self
                .devices
                .lock()
                .unwrap()
                .retain(|device| *device != removed.device),
            _ => {}
        }
        let _ = self.connection().flush();
    }

    fn emit<T: ei::Interface>(&self, capability: DeviceCapability, request: impl Fn(T)) {
        let devices = self.devices.lock().unwrap();
        let Some(device) = devices
            .iter()
            .find(|device| device.has_capability(capability))
        else {
            return;
        };
        request(device.interface::<T>().unwrap());
        let connection = self.connection();
        device.device().frame(
            connection.serial(),
            self.started.elapsed().as_micros() as u64,
        );
        let _ = connection.flush();
    }

    fn connection(&self) -> Connection {
        self.connection.lock().unwrap().clone().unwrap()
    }
}
