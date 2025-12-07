use std::sync::Arc;

use crate::player::{AudioError, SharedAudioBuffer};

#[derive(Debug, Clone)]
pub enum AppEvent {
    Player(PlayerEvent),
    Ui(UiEvent),
}

#[derive(Debug, Clone)]
pub enum PlayerEvent {
    Loaded(SharedAudioBuffer),
    Error(Arc<AudioError>),
    Play,
    Pause,
}

impl From<PlayerEvent> for AppEvent {
    fn from(val: PlayerEvent) -> Self {
        AppEvent::Player(val)
    }
}

#[derive(Debug, Clone)]
pub enum UiEvent {
    LoadSong,
    SongTick,
}

impl From<UiEvent> for AppEvent {
    fn from(val: UiEvent) -> Self {
        AppEvent::Ui(val)
    }
}
