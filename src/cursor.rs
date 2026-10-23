use std::error::Error;

use x11rb::connection::Connection;
use x11rb::protocol::xfixes::ConnectionExt as _;
use x11rb::protocol::xproto::{Atom, AtomEnum, ConnectionExt as _, Window};
use x11rb::rust_connection::RustConnection;

pub struct CursorReader {
    connection: RustConnection,
    root: Window,
    focused_window_atom: Atom,
    cursor_visible_atom: Atom,
    width: u32,
    height: u32,
}

impl CursorReader {
    pub fn connect(display: &str, width: u32, height: u32) -> Result<Self, Box<dyn Error>> {
        let (connection, screen) = x11rb::connect(Some(display))?;
        connection.xfixes_query_version(5, 0)?.reply()?;
        let root = connection.setup().roots[screen].root;
        let focused_window_atom = connection
            .intern_atom(false, b"GAMESCOPE_FOCUSED_WINDOW")?
            .reply()?
            .atom;
        let cursor_visible_atom = connection
            .intern_atom(false, b"GAMESCOPE_CURSOR_VISIBLE_FEEDBACK")?
            .reply()?
            .atom;
        Ok(Self {
            connection,
            root,
            focused_window_atom,
            cursor_visible_atom,
            width,
            height,
        })
    }

    pub fn draw(&self, frame: &mut [u8]) {
        if self.root_cardinal(self.cursor_visible_atom) == Some(0) {
            return;
        }
        let Some(cursor) = self
            .connection
            .xfixes_get_cursor_image()
            .ok()
            .and_then(|cookie| cookie.reply().ok())
        else {
            return;
        };
        let (left, top) = self.screen_position(f32::from(cursor.x), f32::from(cursor.y));
        let (left, top) = (left - i32::from(cursor.xhot), top - i32::from(cursor.yhot));
        for row in 0..i32::from(cursor.height) {
            for column in 0..i32::from(cursor.width) {
                let (x, y) = (left + column, top + row);
                if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
                    continue;
                }
                let pixel = cursor.cursor_image[(row * i32::from(cursor.width) + column) as usize];
                let offset = ((y as u32 * self.width + x as u32) * 4) as usize;
                blend(pixel, &mut frame[offset..offset + 3]);
            }
        }
    }

    fn screen_position(&self, x: f32, y: f32) -> (i32, i32) {
        let (output_width, output_height) = (self.width as f32, self.height as f32);
        let (window_x, window_y, window_width, window_height) =
            self.focused_window()
                .unwrap_or((0.0, 0.0, output_width, output_height));
        let scale = (output_width / window_width).min(output_height / window_height);
        let left = (x - window_x) * scale + (output_width - window_width * scale) / 2.0;
        let top = (y - window_y) * scale + (output_height - window_height * scale) / 2.0;
        (left as i32, top as i32)
    }

    fn root_cardinal(&self, atom: Atom) -> Option<u32> {
        self.connection
            .get_property(false, self.root, atom, AtomEnum::CARDINAL, 0, 1)
            .ok()?
            .reply()
            .ok()?
            .value32()?
            .next()
    }

    fn focused_window(&self) -> Option<(f32, f32, f32, f32)> {
        let window = self.root_cardinal(self.focused_window_atom)?;
        let geometry = self.connection.get_geometry(window).ok()?.reply().ok()?;
        let origin = self
            .connection
            .translate_coordinates(window, self.root, 0, 0)
            .ok()?
            .reply()
            .ok()?;
        Some((
            f32::from(origin.dst_x),
            f32::from(origin.dst_y),
            f32::from(geometry.width),
            f32::from(geometry.height),
        ))
    }
}

fn blend(pixel: u32, destination: &mut [u8]) {
    let [blue, green, red, alpha] = pixel.to_le_bytes();
    for (channel, source) in destination.iter_mut().zip([blue, green, red]) {
        *channel = (u16::from(source) + u16::from(*channel) * u16::from(255 - alpha) / 255) as u8;
    }
}
