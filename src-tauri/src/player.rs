use arc_swap::ArcSwap;
use atomic_float::{AtomicF32, AtomicF64};
use crossbeam_channel::Sender;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

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
    is_playing: Arc<AtomicBool>,
    position: Arc<AtomicF64>,
    volume: Arc<AtomicF32>,
    playback_speed: Arc<AtomicF32>,
}

impl Default for PlayerProps {
    fn default() -> Self {
        Self {
            is_playing: Arc::new(AtomicBool::new(false)),
            position: Arc::new(AtomicF64::new(0.0)),
            volume: Arc::new(AtomicF32::new(0.4)),
            playback_speed: Arc::new(AtomicF32::new(0.97)),
        }
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
    samples: Arc<Vec<f32>>,
}

impl Default for SharedAudioBuffer {
    fn default() -> Self {
        Self {
            sample_rate: SAMPLE_RATE,
            samples: Arc::new(Vec::new()),
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

    pub fn get_position(&self) -> f64 {
        self.props.position.load(Ordering::Relaxed)
    }

    pub fn get_sample_rate(&self) -> u32 {
        self.sample_rate
    }
}
