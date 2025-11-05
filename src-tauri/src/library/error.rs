use serde::Serialize;

#[derive(Debug, thiserror::Error, Serialize)]
pub enum LibraryError {
    #[error("{0}")]
    Io(
        #[serde(skip)]
        #[from]
        std::io::Error,
    ),

    #[error("{0}")]
    Tauri(
        #[serde(skip)]
        #[from]
        tauri::Error,
    ),
}
