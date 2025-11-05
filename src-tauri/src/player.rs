use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use arc_swap::ArcSwap;
use atomic_float::{AtomicF32, AtomicF64};
use crossbeam_channel::Sender;
use serde::Serialize;
use ts_rs::TS;

mod audio_loop;
mod bus;
mod decoder;
mod device;
mod error;
mod event;
mod resample;

pub mod ipc;

pub use decoder::{DecodingError, decode_samples};
pub use error::*;
pub use event::*;

use bus::Bus;
use device::SAMPLE_RATE;
use event::AudioEvent;

#[derive(Debug)]
pub struct PlayerProps {
    pub sample_rate: u32,
    pub is_playing: Arc<AtomicBool>,
    pub position: Arc<AtomicF64>,
    pub volume: Arc<AtomicF32>,
    pub playback_speed: Arc<AtomicF32>,
}

impl Default for PlayerProps {
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            is_playing: Arc::new(AtomicBool::new(false)),
            position: Arc::new(AtomicF64::new(0.0)),
            volume: Arc::new(AtomicF32::new(0.4)),
            playback_speed: Arc::new(AtomicF32::new(0.97)),
        }
    }
}

impl PlayerProps {
    pub fn get_playback_rate(&self, sample_rate: u32) -> f64 {
        let speed = self.playback_speed.load(Ordering::Relaxed) as f64;

        (sample_rate as f64 / self.sample_rate as f64) * speed
    }
}

#[derive(Debug)]
pub struct AudioController {
    _bus: Arc<Bus>,
    shared_audio: Arc<ArcSwap<SharedAudioBuffer>>,
    event_sender: Sender<AudioEvent>,
    sample_rate: u32,
    props: Arc<PlayerProps>,
}

#[derive(Debug, Clone)]
struct SharedAudioBuffer {
    sample_rate: u32,
    channels: Arc<Vec<Vec<f32>>>,
}

impl SharedAudioBuffer {
    pub fn duration(&self) -> usize {
        self.channels.first().map(|s| s.len()).unwrap_or(0)
    }
}

impl Default for SharedAudioBuffer {
    fn default() -> Self {
        Self {
            sample_rate: SAMPLE_RATE,
            channels: Arc::new(Vec::new()),
        }
    }
}

impl AudioController {
    pub fn send_event(&self, msg: impl Into<AudioEvent>) -> Result<(), AudioError> {
        let event = msg.into();

        if let AudioEvent::Playback(event) = &event {
            self.on_atomic_event(*event);
            return Ok(());
        }

        self.event_sender.send(event)?;
        Ok(())
    }

    pub fn on_atomic_event(&self, event: AtomicEvent) {
        match event {
            AtomicEvent::Play => self.props.is_playing.store(true, Ordering::Relaxed),
            AtomicEvent::Pause => self.props.is_playing.store(false, Ordering::Relaxed),
            AtomicEvent::Stop => {
                self.props.is_playing.store(false, Ordering::Relaxed);
                self.props.position.store(0.0, Ordering::Relaxed);
            }
            AtomicEvent::SetVolume(volume) => self.props.volume.store(volume, Ordering::Relaxed),
            AtomicEvent::SetSpeed(speed) => {
                self.props.playback_speed.store(speed, Ordering::Relaxed)
            }
        };
    }

    pub fn get_playback_rate(&self) -> f64 {
        self.props
            .get_playback_rate(self.shared_audio.load().sample_rate)
    }

    pub fn set_position(&self, seconds: usize) {
        let rate = self.get_playback_rate();
        let position = seconds as f64 * rate * self.sample_rate as f64;

        self.props.position.store(position, Ordering::SeqCst);
    }

    pub fn get_position(&self) -> f64 {
        self.props.position.load(Ordering::Relaxed)
    }

    pub fn serialize_props(&self) -> PlayerPropsSerialize {
        PlayerPropsSerialize {
            is_playing: self.props.is_playing.load(Ordering::Relaxed),
            position: self.props.position.load(Ordering::Relaxed),
            volume: self.props.volume.load(Ordering::Relaxed),
            sample_rate: self.sample_rate,
            playback_speed: self.props.playback_speed.load(Ordering::Relaxed),
            local_duraion: self.shared_audio.load().duration(),
            local_sample_rate: self.shared_audio.load().sample_rate,
        }
    }
}

#[derive(Serialize, TS)]
#[ts(export, rename = "PlayerProps")]
pub struct PlayerPropsSerialize {
    #[serde(rename = "isPlaying")]
    pub is_playing: bool,
    pub position: f64,
    pub volume: f32,

    #[serde(rename = "sampleRate")]
    pub sample_rate: u32,
    #[serde(rename = "playbackSpeed")]
    pub playback_speed: f32,

    #[serde(rename = "localDuration")]
    pub local_duraion: usize,
    #[serde(rename = "localSampleRate")]
    pub local_sample_rate: u32,
}
