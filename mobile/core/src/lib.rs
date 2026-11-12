mod av1;
mod bits;
mod h264;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use adamo::video_receive_session::AccessUnitReceiver;
use adamo::{AudioReceiveOptions, AudioReceiver, Joy, Priority, PublishOptions, Rumble, Session, Subscriber};

uniffi::setup_scaffolding!();

#[derive(uniffi::Enum)]
pub enum VideoFormat {
    Av1 {
        width: u32,
        height: u32,
        codec_configuration: Vec<u8>,
    },
    H264 {
        width: u32,
        height: u32,
        sequence_parameter_set: Vec<u8>,
        picture_parameter_set: Vec<u8>,
    },
}

#[uniffi::export]
pub fn video_format(keyframe: Vec<u8>) -> Option<VideoFormat> {
    if h264::is_annex_b(&keyframe) {
        h264::video_format(&keyframe)
    } else {
        av1::video_format(&keyframe)
    }
}

#[uniffi::export]
pub fn h264_length_prefixed(access_unit: Vec<u8>) -> Vec<u8> {
    h264::length_prefixed(&access_unit)
}

const VIDEO_TRACK: &str = "screen";
const AUDIO_TRACK: &str = "audio";

#[derive(Debug, thiserror::Error, uniffi::Error)]
#[uniffi(flat_error)]
pub enum PlayError {
    #[error("{message}")]
    Failed { message: String },
}

impl From<adamo::Error> for PlayError {
    fn from(error: adamo::Error) -> Self {
        Self::Failed {
            message: error.to_string(),
        }
    }
}

#[derive(uniffi::Record)]
pub struct VideoFrame {
    pub data: Vec<u8>,
    pub is_keyframe: bool,
}

#[derive(uniffi::Record)]
pub struct AudioFrame {
    pub pcm_bytes: Vec<u8>,
    pub sample_rate: u32,
    pub channel_count: u16,
}

#[derive(uniffi::Record)]
pub struct GamepadRumble {
    pub gamepad_index: u8,
    pub strong_magnitude: f32,
    pub weak_magnitude: f32,
    pub duration_ms: u32,
}

#[derive(uniffi::Object)]
pub struct Connection {
    session: Arc<Session>,
    robot: String,
    video: Mutex<AccessUnitReceiver>,
    audio: AudioReceiver,
    rumbles: Subscriber<'static>,
}

#[uniffi::export]
impl Connection {
    #[uniffi::constructor]
    pub fn connect(address: String, pairing_key: String, screen_width: u32, screen_height: u32) -> Result<Arc<Self>, PlayError> {
        let session = Arc::new(Session::connect_local(&address, &pairing_key)?);
        let robot = session.local_robot().unwrap().to_owned();
        let video = AccessUnitReceiver::subscribe(session.clone(), &robot, VIDEO_TRACK)?;
        let audio = AudioReceiver::subscribe(&session, &robot, AUDIO_TRACK, AudioReceiveOptions::default())?;
        let rumbles = session.subscribe_owned(&format!("{robot}/feedback/gamepad/rumble"))?;
        let display = serde_json::json!({"width": screen_width, "height": screen_height});
        session.put(&format!("{robot}/control/json/display"), display.to_string().as_bytes(), PublishOptions::default())?;
        Ok(Arc::new(Self {
            session,
            robot,
            video: Mutex::new(video),
            audio,
            rumbles,
        }))
    }

    pub fn receive_video(&self, timeout_ms: u32) -> Result<Option<VideoFrame>, PlayError> {
        let access_unit = self.video.lock().unwrap().recv(milliseconds(timeout_ms))?;
        Ok(access_unit.map(|access_unit| VideoFrame {
            data: access_unit.payload,
            is_keyframe: access_unit.keyframe,
        }))
    }

    pub fn request_keyframe(&self) -> Result<(), PlayError> {
        Ok(self.video.lock().unwrap().request_keyframe()?)
    }

    pub fn receive_audio(&self, timeout_ms: u32) -> Result<Option<AudioFrame>, PlayError> {
        let frame = self.audio.recv(milliseconds(timeout_ms))?;
        Ok(frame.map(|frame| AudioFrame {
            pcm_bytes: frame.samples.iter().flat_map(|sample| sample.to_le_bytes()).collect(),
            sample_rate: frame.sample_rate,
            channel_count: frame.channels,
        }))
    }

    pub fn receive_rumble(&self, timeout_ms: u32) -> Result<Option<GamepadRumble>, PlayError> {
        let sample = match self.rumbles.recv(Some(milliseconds(timeout_ms))) {
            Ok(sample) => sample,
            Err(adamo::Error::Timeout) => return Ok(None),
            Err(error) => return Err(error.into()),
        };
        let rumble: Rumble = serde_json::from_slice(&sample.payload).map_err(|error| PlayError::Failed {
            message: error.to_string(),
        })?;
        Ok(Some(GamepadRumble {
            gamepad_index: rumble.gamepad_index,
            strong_magnitude: rumble.strong_magnitude,
            weak_magnitude: rumble.weak_magnitude,
            duration_ms: rumble.duration_ms,
        }))
    }

    pub fn send_gamepad(&self, gamepad_index: u8, axes: Vec<f64>, buttons: Vec<i32>) -> Result<(), PlayError> {
        let payload = Joy { axes, buttons, stamp: 0.0 }.to_msgpack()?;
        let options = PublishOptions {
            priority: Priority::REAL_TIME,
            express: true,
            compression: false,
        };
        Ok(self.session.put(&format!("{}/control/joy/{gamepad_index}", self.robot), &payload, options)?)
    }
}

fn milliseconds(timeout_ms: u32) -> Duration {
    Duration::from_millis(timeout_ms.into())
}
