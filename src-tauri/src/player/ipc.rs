use serde::Serialize;
use std::sync::atomic::Ordering;
use std::{path::PathBuf, sync::Arc};
use tauri::State;
use ts_rs::TS;

use crate::player::PlayerProps;

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
pub fn get_position(player: State<AudioController>) -> f64 {
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
    speed: f32,
) -> Result<(), AudioError> {
    player.send_event(AtomicEvent::SetSpeed(speed))
}

impl From<&PlayerProps> for PlayerPropsSerialize {
    fn from(value: &PlayerProps) -> Self {
        Self {
            is_playing: value.is_playing.load(Ordering::Relaxed),
            position: value.position.load(Ordering::Relaxed),
            volume: value.volume.load(Ordering::Relaxed),
            playback_speed: value.playback_speed.load(Ordering::Relaxed),
        }
    }
}

#[derive(Serialize, TS)]
#[ts(export)]
#[ts(rename = "PlayerProps")]
pub struct PlayerPropsSerialize {
    #[serde(rename = "isPlaying")]
    pub is_playing: bool,
    pub position: f64,
    pub volume: f32,
    #[serde(rename = "playbackSpeed")]
    pub playback_speed: f32,
}

#[tauri::command]
pub fn player_get_props(player: State<AudioController>) -> PlayerPropsSerialize {
    player.props.as_ref().into()
}
