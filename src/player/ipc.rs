use serde::Serialize;
use std::{path::PathBuf, sync::Arc};
use tauri::State;
use ts_rs::TS;

use super::{AtomicEvent, PlayerPropsSerialize};
use super::{AudioController, AudioError, SharedAudioBuffer, decode_samples};

#[derive(Serialize, TS)]
#[ts(export, rename = "LoadSongResult")]
pub struct LoadSongSerialize {
    #[serde(rename = "sampleRate")]
    sample_rate: u32,
    duration: usize,
}

impl From<&SharedAudioBuffer> for LoadSongSerialize {
    fn from(value: &SharedAudioBuffer) -> Self {
        Self {
            duration: value.duration(),
            sample_rate: value.sample_rate,
        }
    }
}

#[tauri::command]
pub fn player_load_song(
    player: State<AudioController>,
    path: PathBuf,
) -> Result<LoadSongSerialize, AudioError> {
    let buf: SharedAudioBuffer = decode_samples(&path)?.into();
    let result = LoadSongSerialize::from(&buf);

    player.shared_audio.swap(Arc::new(buf));

    Ok(result)
}

#[tauri::command]
pub fn player_get_position(player: State<AudioController>) -> f64 {
    player.get_position()
}

#[tauri::command]
pub fn player_set_position(player: State<AudioController>, pos: usize) {
    println!("ipc {pos}");
    player.set_position(pos);
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
    speed: f32,
) -> Result<(), AudioError> {
    player.send_event(AtomicEvent::SetSpeed(speed))
}

#[tauri::command]
pub fn player_get_props(player: State<AudioController>) -> PlayerPropsSerialize {
    player.serialize_props()
}
