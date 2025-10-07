use serde::Serialize;

use super::DecodingError;

#[derive(Debug, thiserror::Error, Serialize)]
pub enum AudioError {
    #[error("{0}")]
    DecodingError(
        #[serde(skip)]
        #[from]
        DecodingError,
    ),
}
