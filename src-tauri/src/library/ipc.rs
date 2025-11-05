use std::fs;

use tauri::{AppHandle, Manager};

use super::{LibraryEntry, LibraryError};

#[tauri::command]
pub fn load_library(
    app_handle: AppHandle,
    directory: Option<&str>,
) -> Result<Vec<LibraryEntry>, LibraryError> {
    let base_dir = app_handle.path().audio_dir()?;
    let base_dir = directory.map(|d| d.into()).unwrap_or_else(|| base_dir);

    let entries = fs::read_dir(base_dir)?;

    Ok(entries
        .filter_map(Result::ok)
        .filter_map(|e| LibraryEntry::try_from(e).ok())
        .collect())
}
