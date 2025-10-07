use arc_swap::ArcSwap;
use std::sync::{atomic::AtomicUsize, Arc};

mod audio_loop;
mod bus;
mod decoder;
mod device;
mod error;
mod resample;

pub mod ipc;

pub use decoder::{decode_samples, DecodingError};
pub use error::AudioError;

use bus::Bus;
use device::SAMPLE_RATE;

#[derive(Debug, Default)]
pub struct AudioController {
    shared_audio: Arc<ArcSwap<SharedAudioBuffer>>,
    bus: Arc<Bus>,
}

#[derive(Debug, Clone)]
struct SharedAudioBuffer {
    samples: Arc<Vec<f32>>,
    _sample_rate: u32,
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
