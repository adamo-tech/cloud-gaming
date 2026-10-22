use std::error::Error;
use std::sync::Arc;
use std::time::{Duration, Instant};

use adamo::{Priority, PublishOptions, Robot};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde_json::json;
use x11rb::protocol::xfixes::ConnectionExt;

const POLL_INTERVAL: Duration = Duration::from_millis(8);
const IMAGE_REPEAT_INTERVAL: Duration = Duration::from_secs(1);

pub fn publish_cursor(robot: Arc<Robot>, display: &str, width: u32, height: u32) -> Result<(), Box<dyn Error>> {
    let (connection, _) = x11rb::connect(Some(display))?;
    connection.xfixes_query_version(5, 0)?.reply()?;
    let options = PublishOptions { priority: Priority::REAL_TIME, express: true, compression: false };
    let image_key = format!("{}/cursor/image", robot.name());
    let position_key = format!("{}/cursor/position", robot.name());
    std::thread::spawn(move || {
        let mut last_serial = None;
        let mut last_image_sent = Instant::now();
        let mut last_position = None;
        while let Some(cursor) = connection.xfixes_get_cursor_image().ok().and_then(|cookie| cookie.reply().ok()) {
            if last_serial != Some(cursor.cursor_serial) || last_image_sent.elapsed() >= IMAGE_REPEAT_INTERVAL {
                let rgba: Vec<u8> = cursor.cursor_image.iter().flat_map(|pixel| {
                    let [blue, green, red, alpha] = pixel.to_le_bytes();
                    let unpremultiply = |channel: u8| if alpha == 0 { 0 } else { (u16::from(channel) * 255 / u16::from(alpha)) as u8 };
                    [unpremultiply(red), unpremultiply(green), unpremultiply(blue), alpha]
                }).collect();
                let image = json!({ "width": cursor.width, "height": cursor.height, "hot_x": cursor.xhot, "hot_y": cursor.yhot, "rgba": STANDARD.encode(rgba) });
                let _ = robot.put(&image_key, image.to_string().as_bytes(), options);
                last_serial = Some(cursor.cursor_serial);
                last_image_sent = Instant::now();
            }
            if last_position != Some((cursor.x, cursor.y)) {
                let position = json!({ "x": cursor.x, "y": cursor.y, "width": width, "height": height });
                let _ = robot.put(&position_key, position.to_string().as_bytes(), options);
                last_position = Some((cursor.x, cursor.y));
            }
            std::thread::sleep(POLL_INTERVAL);
        }
    });
    Ok(())
}
