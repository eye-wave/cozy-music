use std::{
    path::Path,
    process::Command,
    sync::{atomic::AtomicUsize, Arc},
};

use crate::player::SharedAudioBuffer;

mod opus;
mod sym;

#[derive(Debug, thiserror::Error)]
pub enum DecodingError {
    #[error("")]
    NoTrack,

    #[error("Unsupported format {0}.")]
    UnsupportedFormat(String),

    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Opus(#[from] ogg_opus::Error),

    #[error("{0}")]
    Symphonia(#[from] symphonia::core::errors::Error),

    #[error("Failed to decode {0} file.")]
    Path(String),
}

pub struct DecoderResult {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
}

impl From<DecoderResult> for SharedAudioBuffer {
    fn from(value: DecoderResult) -> Self {
        Self {
            _sample_rate: value.sample_rate,
            pos: Arc::new(AtomicUsize::new(0)),
            samples: Arc::new(value.samples),
        }
    }
}

pub(super) type DecodingResult = std::result::Result<DecoderResult, DecodingError>;

fn get_mime_type<P: AsRef<Path>>(path: &P) -> Result<String, DecodingError> {
    let output = Command::new("xdg-mime")
        .args(["query", "filetype", &path.as_ref().to_string_lossy()])
        .output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(DecodingError::Path(
            path.as_ref().to_string_lossy().to_string(),
        ))
    }
}

pub fn decode_samples<P: AsRef<Path>>(path: &P) -> DecodingResult {
    let mime = get_mime_type(&path)?;

    match mime.as_ref() {
        "audio/x-opus+ogg" => opus::decode_audio(&path),
        "audio/aac" | "audio/flac" | "audio/mp2" | "audio/mp4" | "audio/mpeg" | "audio/x-aiff"
        | "audio/x-caf" | "audio/x-vorbis+ogg" | "audio/x-wav" => sym::decode_audio(&path),
        mime => Err(DecodingError::UnsupportedFormat(mime.to_string())),
    }
}
