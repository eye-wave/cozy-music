use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use arc_swap::ArcSwap;
use atomic_float::{AtomicF32, AtomicF64};
use crossbeam_channel::Sender;

mod audio_loop;
mod bus;
mod decoder;
mod device;
mod error;
mod resample;

pub mod event;

pub use decoder::*;
pub use error::*;

use bus::Bus;
use device::SAMPLE_RATE;
use event::{AtomicEvent, AudioEvent};

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
    pub shared_audio: Arc<ArcSwap<SharedAudioBuffer>>,
    event_sender: Sender<AudioEvent>,
    props: Arc<PlayerProps>,
}

#[derive(Debug, Clone)]
pub struct SharedAudioBuffer {
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

    pub fn sample_rate(&self) -> u32 {
        self.props.sample_rate
    }

    pub fn get_song_position(&self) -> f64 {
        self.props.position.load(Ordering::Relaxed)
    }

    pub fn get_song_position_string(&self) -> String {
        let buffer = self.shared_audio.load();

        let duration = buffer.duration();
        let pos = self.get_song_position();

        let (dm, ds) = format_sample_time(duration as f64, buffer.sample_rate);
        let (pm, ps) = format_sample_time(pos, self.sample_rate());

        format!("{dm:02}:{ds:02}/{pm:02}:{ps:02}")
    }

    pub fn get_is_playing(&self) -> bool {
        self.props.is_playing.load(Ordering::Relaxed)
    }

    pub fn set_position(&self, pos: usize) {
        let rate = self.sample_rate() as f64;
        let position = rate * pos as f64;

        self.props.position.store(position, Ordering::SeqCst);
    }
}

fn format_sample_time(samples: f64, sample_rate: u32) -> (usize, usize) {
    let total_secs = samples / sample_rate as f64;
    let min = (total_secs / 60.0).floor() as usize;
    let s = total_secs - (min as f64 * 60.0);

    (min, s as usize)
}
