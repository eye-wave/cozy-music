use cfg_if::cfg_if;
use std::{path::Path, process::Command, sync::Arc};

use crate::player::SharedAudioBuffer;

#[cfg(feature = "opus")]
mod opus;
mod sym;

#[derive(Debug, thiserror::Error, serde::Serialize)]
pub enum DecodingError {
    #[error("")]
    NoTrack,

    #[error("Unsupported format {0}.")]
    UnsupportedFormat(String),

    #[error("{0}")]
    Io(
        #[from]
        #[serde(skip)]
        std::io::Error,
    ),

    #[cfg(feature = "opus")]
    #[error("{0}")]
    Opus(
        #[from]
        #[serde(skip)]
        ogg_opus::Error,
    ),

    #[error("{0}")]
    Symphonia(
        #[from]
        #[serde(skip)]
        symphonia::core::errors::Error,
    ),

    #[error("Failed to decode {0} file.")]
    Path(String),
}

#[derive(Debug, Clone)]
pub struct DecoderResult {
    pub channels: Arc<Vec<Vec<f32>>>,
    pub sample_rate: u32,
}

impl From<DecoderResult> for SharedAudioBuffer {
    fn from(value: DecoderResult) -> Self {
        Self {
            sample_rate: value.sample_rate,
            channels: value.channels,
        }
    }
}

pub type DecodingResult = std::result::Result<DecoderResult, DecodingError>;

fn get_mime_type<P>(path: &P) -> Result<String, DecodingError>
where
    P: AsRef<Path> + ?Sized,
{
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

pub fn decode_samples<P>(path: &P) -> DecodingResult
where
    P: AsRef<Path> + ?Sized,
{
    let mime = get_mime_type(&path)?;

    match mime.as_ref() {
        "audio/x-opus+ogg" => {
            cfg_if! {
                if #[cfg(feature="opus")] { opus::decode_audio(&path) }
                else {
                    Err(DecodingError::UnsupportedFormat(mime.to_string()))
                }
            }
        }
        "audio/aac" | "audio/flac" | "audio/mp2" | "audio/mp4" | "audio/mpeg" | "audio/x-aiff"
        | "audio/x-caf" | "audio/x-vorbis+ogg" | "audio/x-wav" | "audio/vnd.wave" => {
            sym::decode_audio(&path)
        }
        mime => Err(DecodingError::UnsupportedFormat(mime.to_string())),
    }
}
