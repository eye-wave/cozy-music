use std::{path::PathBuf, sync::Arc};
use tauri::State;

use super::AtomicEvent;
use super::{AudioController, AudioError, SharedAudioBuffer, decode_samples};

#[tauri::command]
pub fn load_song(player: State<AudioController>, path: PathBuf) -> Result<usize, AudioError> {
    let buf: SharedAudioBuffer = decode_samples(&path)?.into();
    let duration = buf.samples.len();

    player.shared_audio.swap(Arc::new(buf));

    Ok(duration)
}

#[tauri::command]
pub fn get_samplerate(player: State<AudioController>) -> u32 {
    player.get_sample_rate()
}

#[tauri::command]
pub fn get_position(player: State<AudioController>) -> f32 {
    player.get_position()
}

#[tauri::command]
pub fn player_play(player: State<AudioController>) -> Result<(), AudioError> {
    player.send_event(AtomicEvent::Play)
}

#[tauri::command]
pub fn player_pause(player: State<AudioController>) -> Result<(), AudioError> {
    player.send_event(AtomicEvent::Pause)
}

#[tauri::command]
pub fn player_stop(player: State<AudioController>) -> Result<(), AudioError> {
    player.send_event(AtomicEvent::Stop)
}

#[tauri::command]
pub fn player_set_volume(player: State<AudioController>, volume: f32) -> Result<(), AudioError> {
    player.send_event(AtomicEvent::SetVolume(volume))
}

#[tauri::command]
pub fn player_set_playback_speed(
    player: State<AudioController>,
    volume: f32,
) -> Result<(), AudioError> {
    player.send_event(AtomicEvent::SetSpeed(volume))
}
