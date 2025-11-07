use serde::Serialize;

use super::{ChannelError, DecodingError};

#[derive(Debug, thiserror::Error, Serialize)]
pub enum AudioError {
    #[error("{0}")]
    Channel(
        #[from]
        #[serde(skip)]
        ChannelError,
    ),

    #[error("{0}")]
    Decoding(#[from] DecodingError),

    #[error("{0}")]
    Config(#[from] ConfigError),

    #[error("{0}")]
    Stream(#[from] StreamError),
}

#[derive(Debug, thiserror::Error, Serialize)]
pub enum ConfigError {
    #[error("")]
    NoOutputDevice,

    #[error("")]
    ConfigQueryFailed,

    #[error("")]
    NoConfigAvailable,
}

#[derive(Debug, thiserror::Error, Serialize)]
pub enum StreamError {
    #[error("")]
    StreamBuildFailed,

    #[error("")]
    StreamPlayFailed,
}
