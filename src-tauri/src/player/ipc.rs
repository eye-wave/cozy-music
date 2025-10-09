use std::{path::PathBuf, sync::Arc};
use tauri::State;

use super::PlaybackEvent;
use super::{decode_samples, AudioController, AudioError, SharedAudioBuffer};

#[tauri::command]
pub fn load_song(player: State<AudioController>, path: PathBuf) -> Result<(), AudioError> {
    let buf: SharedAudioBuffer = decode_samples(&path)?.into();

    println!("song decoded!");
    player.shared_audio.swap(Arc::new(buf));

    Ok(())
}

#[tauri::command]
pub fn player_play(player: State<AudioController>) -> Result<(), AudioError> {
    player.send_event(PlaybackEvent::Play)
}

#[tauri::command]
pub fn player_pause(player: State<AudioController>) -> Result<(), AudioError> {
    player.send_event(PlaybackEvent::Pause)
}

#[tauri::command]
pub fn player_stop(player: State<AudioController>) -> Result<(), AudioError> {
    player.send_event(PlaybackEvent::Stop)
}
