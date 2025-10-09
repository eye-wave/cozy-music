use super::AudioError;

#[derive(Debug, Clone)]
pub enum AudioEvent {
    Playback(PlaybackEvent),
}

#[derive(Debug, Clone, Copy)]
pub enum PlaybackEvent {
    Play,
    Pause,
    Stop,
}

impl From<PlaybackEvent> for AudioEvent {
    fn from(value: PlaybackEvent) -> Self {
        Self::Playback(value)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ChannelError {
    #[error("{0}")]
    Send(#[from] crossbeam_channel::SendError<AudioEvent>),

    #[error("{0}")]
    Recv(#[from] crossbeam_channel::RecvError),
}

impl From<crossbeam_channel::SendError<AudioEvent>> for AudioError {
    fn from(value: crossbeam_channel::SendError<AudioEvent>) -> Self {
        Self::Channel(ChannelError::Send(value))
    }
}
