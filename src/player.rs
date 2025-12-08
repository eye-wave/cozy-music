use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};

use arc_swap::ArcSwap;
use atomic_float::{AtomicF32, AtomicF64};
use bitflags::bitflags;
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

bitflags! {
    pub struct PlayerFlags: u8 {
        const IS_PLAYING = 1 << 0;
        const LOOP       = 1 << 1;
        const SHUFFLE    = 1 << 2;
        const MUTED      = 1 << 3;
    }
}

#[derive(Debug)]
pub struct PlayerProps {
    pub sample_rate: u32,
    pub flags: AtomicU8,
    pub position: AtomicF64,
    pub volume: AtomicF32,
    pub playback_speed: AtomicF64,
}

impl PlayerProps {
    pub fn set_flag(&self, flag: PlayerFlags, ordering: Ordering) {
        self.flags.fetch_or(flag.bits(), ordering);
    }

    pub fn clear_flag(&self, flag: PlayerFlags, ordering: Ordering) {
        self.flags.fetch_and(!flag.bits(), ordering);
    }

    pub fn get_flag(&self, flag: PlayerFlags, ordering: Ordering) -> bool {
        let bits = self.flags.load(ordering);
        PlayerFlags::from_bits_truncate(bits).contains(flag)
    }
}

impl Default for PlayerProps {
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            flags: AtomicU8::new(0),
            position: AtomicF64::new(0.0),
            volume: AtomicF32::new(0.4),
            playback_speed: AtomicF64::new(0.97),
        }
    }
}

impl PlayerProps {
    pub fn get_playback_rate(&self, sample_rate: u32) -> f64 {
        let speed = self.playback_speed.load(Ordering::Relaxed);

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
    pub fn send_event(&self, msg: impl Into<AudioEvent>) {
        let event = msg.into();

        if let AudioEvent::Playback(event) = &event {
            self.on_atomic_event(*event);
            return;
        }

        self.event_sender.send(event).ok();
    }

    pub fn on_atomic_event(&self, event: AtomicEvent) {
        match event {
            AtomicEvent::Play => self
                .props
                .set_flag(PlayerFlags::IS_PLAYING, Ordering::SeqCst),
            AtomicEvent::Pause => self
                .props
                .clear_flag(PlayerFlags::IS_PLAYING, Ordering::SeqCst),
            AtomicEvent::Stop => {
                self.props
                    .clear_flag(PlayerFlags::IS_PLAYING, Ordering::SeqCst);

                self.props.position.store(0.0, Ordering::SeqCst);
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

    pub fn get_volume(&self) -> f32 {
        self.props.volume.load(Ordering::Relaxed)
    }

    pub fn get_playback_rate(&self) -> f64 {
        let config_sample_rate = self.sample_rate() as f64;
        let sample_rate = self.shared_audio.load().sample_rate as f64;
        let speed = self.props.playback_speed.load(Ordering::Relaxed);

        (sample_rate / config_sample_rate) * speed
    }

    pub fn get_song_duration(&self) -> usize {
        self.shared_audio.load().duration()
    }

    pub fn get_song_position(&self) -> f64 {
        self.props.position.load(Ordering::Relaxed)
    }

    pub fn get_song_position_percent(&self) -> f64 {
        let buffer = self.shared_audio.load();
        let duration_samples = buffer.duration() as f64;
        let pos_samples = self.get_song_position();

        pos_samples / duration_samples
    }

    pub fn get_song_position_pretty(&self) -> [u8; 13] {
        let buffer = self.shared_audio.load();

        let duration = buffer.duration();
        let pos = self.get_song_position();

        let (dm, ds) = format_sample_time(duration as f64, buffer.sample_rate);
        let (pm, ps) = format_sample_time(pos, self.sample_rate());

        let mut buffer = *b"00:00 / 00:00";

        buffer[0..2].copy_from_slice(&pad_start(dm));
        buffer[3..5].copy_from_slice(&pad_start(ds));

        buffer[8..10].copy_from_slice(&pad_start(pm));
        buffer[11..13].copy_from_slice(&pad_start(ps));

        buffer
    }

    pub fn get_is_playing(&self) -> bool {
        self.props
            .get_flag(PlayerFlags::IS_PLAYING, Ordering::Relaxed)
    }

    pub fn get_speed(&self) -> f64 {
        self.props.playback_speed.load(Ordering::Relaxed)
    }

    pub fn set_position(&self, pos_percent: f64) {
        let duration = self.shared_audio.load().duration() as f64;
        let rate = self.get_playback_rate();
        let position = pos_percent * rate * duration;

        self.props.position.store(position, Ordering::SeqCst);
    }
}

fn format_sample_time(samples: f64, sample_rate: u32) -> (u8, u8) {
    let total_secs = samples / sample_rate as f64;
    let min = (total_secs / 60.0).floor() as u8;
    let s = total_secs - (min as f64 * 60.0);

    (min, s as u8)
}

fn pad_start(num: u8) -> [u8; 2] {
    [((num / 10) % 10) + 48, (num % 10) + 48]
}
