use std::fs::{self, DirEntry};

use serde::Serialize;
use ts_rs::TS;

mod error;

pub mod ipc;

pub use error::*;

#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub enum LibraryEntry {
    File { name: String },
    Folder { name: String, items: usize },
}

impl TryFrom<DirEntry> for LibraryEntry {
    type Error = std::io::Error;

    fn try_from(value: DirEntry) -> Result<Self, Self::Error> {
        let is_file = value.path().is_file();
        let name = value.file_name().to_string_lossy().to_string();

        match is_file {
            true => Ok(Self::File { name }),
            false => {
                let items = fs::read_dir(value.path())?.count();

                Ok(Self::Folder { name, items })
            }
        }
    }
}
