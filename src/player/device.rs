use arc_swap::ArcSwap;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crossbeam_channel::bounded;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use super::error::*;

use crate::player::audio_loop::{AudioLoopState, build_stream_match};
use crate::player::bus::Bus;
use crate::player::{PlayerProps, SharedAudioBuffer};

use super::AudioController;

pub const SAMPLE_RATE: u32 = 44_100;

impl AudioController {
    pub fn create() -> Result<Self, AudioError> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or(ConfigError::NoOutputDevice)?;

        let mut supported_configs = device
            .supported_output_configs()
            .map_err(|_| ConfigError::ConfigQueryFailed)?;

        let config: cpal::StreamConfig = pick_config(&mut supported_configs)
            .ok_or(ConfigError::NoConfigAvailable)?
            .into();

        let shared_audio = Arc::new(ArcSwap::from_pointee(SharedAudioBuffer::default()));
        let bus = Arc::new(Bus::default());
        let (tx, rx) = bounded(128);
        let rx = Arc::new(rx);

        let sample_rate: u32 = config.sample_rate.0;
        let props = Arc::new(PlayerProps {
            sample_rate,
            ..Default::default()
        });

        let state = AudioLoopState {
            rx: Arc::clone(&rx),
            bus: Arc::clone(&bus),
            shared: Arc::clone(&shared_audio),
            props: Arc::clone(&props),
        };

        let stream = build_stream_match!(
            device,
            state.clone(),
            &config,
            state_for_thread,
            |err| eprintln!("Audio stream error: {err}"),
            {
                cpal::SampleFormat::F32 => f32,
                cpal::SampleFormat::I16 => i16,
                cpal::SampleFormat::I24 => cpal::I24,
                cpal::SampleFormat::I32 => i32,
                cpal::SampleFormat::I8 => i8,
                cpal::SampleFormat::U16 => u16,
                cpal::SampleFormat::U32 => u32,
                cpal::SampleFormat::U8 => u8,
            }
        )
        .map_err(|_| StreamError::StreamBuildFailed)?;

        stream.play().map_err(|_| StreamError::StreamPlayFailed)?;

        thread::spawn(move || {
            let _keep_alive = stream;
            loop {
                thread::sleep(Duration::from_secs(60));
            }
        });

        Ok(AudioController {
            _bus: bus,
            event_sender: tx,
            shared_audio,
            props,
        })
    }
}

fn pick_config(configs: &mut cpal::SupportedOutputConfigs) -> Option<cpal::SupportedStreamConfig> {
    let configs: Vec<_> = configs.collect();
    const PREFERRED_RATE: u32 = 44_100;

    // 1. Stereo F32 at 44.1k
    if let Some(config) = configs
        .iter()
        .filter(|c| c.channels() >= 2 && c.sample_format() == cpal::SampleFormat::F32)
        .find(|c| {
            c.min_sample_rate().0 <= PREFERRED_RATE && c.max_sample_rate().0 >= PREFERRED_RATE
        })
    {
        return Some(config.with_sample_rate(cpal::SampleRate(PREFERRED_RATE)));
    }

    // 2. Stereo F32 at any rate
    if let Some(config) = configs
        .iter()
        .find(|c| c.channels() >= 2 && c.sample_format() == cpal::SampleFormat::F32)
    {
        return Some(config.with_sample_rate(cpal::SampleRate(config.max_sample_rate().0)));
    }

    // 3. Stereo any type at any rate
    if let Some(config) = configs.iter().find(|c| c.channels() >= 2) {
        return Some(config.with_sample_rate(cpal::SampleRate(config.max_sample_rate().0)));
    }

    // 4. Mono fallback
    configs
        .first()
        .map(|c| c.with_sample_rate(cpal::SampleRate(c.max_sample_rate().0)))
}
