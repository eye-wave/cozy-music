use arc_swap::ArcSwap;
use crossbeam_channel::Sender;
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc,
};

mod audio_loop;
mod bus;
mod decoder;
mod device;
mod error;
mod event;
mod resample;

pub mod ipc;

pub use decoder::{decode_samples, DecodingError};
pub use error::*;
pub use event::*;

use bus::Bus;
use device::SAMPLE_RATE;
use event::AudioEvent;

#[derive(Debug)]
pub struct AudioController {
    _bus: Arc<Bus>,
    is_playing: Arc<AtomicBool>,
    shared_audio: Arc<ArcSwap<SharedAudioBuffer>>,
    event_sender: Sender<AudioEvent>,
}

#[derive(Debug, Clone)]
struct SharedAudioBuffer {
    _sample_rate: u32,
    samples: Arc<Vec<f32>>,
    pos: Arc<AtomicUsize>,
}

impl Default for SharedAudioBuffer {
    fn default() -> Self {
        Self {
            samples: Arc::new(Vec::new()),
            _sample_rate: SAMPLE_RATE,
            pos: Arc::new(AtomicUsize::new(0)),
        }
    }
}

impl AudioController {
    pub fn send_event(&self, msg: impl Into<AudioEvent>) -> Result<(), AudioError> {
        let event = msg.into();

        match &event {
            AudioEvent::Playback(event) => self.on_atomic_event(*event),
        }

        self.event_sender.send(event)?;

        Ok(())
    }

    pub fn on_atomic_event(&self, event: PlaybackEvent) {
        match event {
            PlaybackEvent::Play => {
                self.is_playing.swap(true, Ordering::Relaxed);
            }
            PlaybackEvent::Pause => {
                self.is_playing.swap(false, Ordering::Relaxed);
            }
            PlaybackEvent::Stop => {
                self.is_playing.swap(false, Ordering::Relaxed);
                self.shared_audio.load().pos.swap(0, Ordering::Relaxed);
            }
        };
    }
}
