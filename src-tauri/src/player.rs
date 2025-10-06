mod decoder;
mod device;
mod resample;

use decoder::DecodingError;

pub use decoder::decode_samples;

#[derive(Debug, Default)]
pub struct AudioControler;

#[derive(Debug, thiserror::Error)]
pub enum AudioError {
    #[error("{0}")]
    DecodingError(#[from] DecodingError),
}
