use std::{path::PathBuf, sync::Arc};

use tauri::State;

use super::{decode_samples, AudioController, AudioError, SharedAudioBuffer};

#[tauri::command]
pub fn load_song(player: State<AudioController>, path: PathBuf) -> Result<(), AudioError> {
    let buf: SharedAudioBuffer = decode_samples(&path)?.into();

    println!("song decoded!");
    player.shared_audio.swap(Arc::new(buf));

    Ok(())
}
