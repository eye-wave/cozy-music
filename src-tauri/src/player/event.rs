use super::AudioError;

#[allow(unused)]
#[derive(Debug, Clone)]
pub enum AudioEvent {
    Playback(AtomicEvent),

    Placeholder,
}

#[derive(Debug, Clone, Copy)]
pub enum AtomicEvent {
    Play,
    Pause,
    Stop,
    SetVolume(f32),
    SetSpeed(f32),
}

impl From<AtomicEvent> for AudioEvent {
    fn from(value: AtomicEvent) -> Self {
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
