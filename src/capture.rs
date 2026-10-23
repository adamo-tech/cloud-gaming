use std::error::Error;

use adamo::{Rate, VideoTrack};
use gstreamer::prelude::*;
use gstreamer_app::AppSink;

use crate::cursor::CursorReader;

pub fn stream_gamescope(
    track: VideoTrack,
    cursor: CursorReader,
    width: u32,
    height: u32,
    fps: u32,
) -> Result<(), Box<dyn Error>> {
    gstreamer::init()?;
    let description = [
        "pipewiresrc target-object=gamescope always-copy=true keepalive-time=100",
        &format!("video/x-raw,format=BGRx,width={width},height={height}"),
        "appsink name=sink max-buffers=1 drop=true sync=false",
    ]
    .join(" ! ");
    let pipeline = gstreamer::parse::launch(&description)?
        .downcast::<gstreamer::Pipeline>()
        .map_err(|_| "the capture pipeline is not a pipeline")?;
    let sink = pipeline
        .by_name("sink")
        .ok_or("the capture pipeline has no sink")?
        .downcast::<AppSink>()
        .map_err(|_| "the capture sink is not an appsink")?;
    pipeline.set_state(gstreamer::State::Playing)?;
    std::thread::spawn(move || {
        let mut rate = Rate::new(f64::from(fps));
        let mut latest = vec![0; (width * height * 4) as usize];
        loop {
            if let Some(buffer) = sink
                .try_pull_sample(gstreamer::ClockTime::ZERO)
                .and_then(|sample| sample.buffer_owned())
                && let Ok(map) = buffer.map_readable()
            {
                let size = latest.len();
                latest.copy_from_slice(&map[..size]);
            }
            let mut frame = latest.clone();
            cursor.draw(&mut frame);
            if track.send_owned(frame).is_err() {
                break;
            }
            rate.sleep();
        }
        let _ = pipeline.set_state(gstreamer::State::Null);
    });
    Ok(())
}
