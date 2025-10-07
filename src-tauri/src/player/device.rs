use arc_swap::ArcSwap;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::player::audio_loop::build_stream_match;
use crate::player::bus::Bus;
use crate::player::SharedAudioBuffer;

use super::AudioController;

pub const SAMPLE_RATE: u32 = 44_100;

impl AudioController {
    pub fn new() -> Self {
        let host = cpal::default_host();
        let device = host.default_output_device().expect("No output device");
        let mut supported_configs = device
            .supported_output_configs()
            .expect("Error getting configs");

        let config = pick_config(&mut supported_configs);
        if config.is_none() {
            return Self::default();
        }

        let shared_audio = Arc::new(ArcSwap::from_pointee(SharedAudioBuffer::default()));
        let bus = Arc::new(Bus::default());

        let shared_audio_ref = Arc::clone(&shared_audio);

        let stream = build_stream_match!(
            device,
            &shared_audio_ref,
            &bus,
            &config.unwrap().into(),
            state_for_thread,
            |err| eprintln!("{err}"),
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
        .expect("Failed to build output stream");

        stream.play().unwrap();

        thread::spawn(|| {
            let _stream = stream;

            loop {
                thread::sleep(Duration::from_secs(60));
            }
        });

        Self { bus, shared_audio }
    }
}
fn pick_config(configs: &mut cpal::SupportedOutputConfigs) -> Option<cpal::SupportedStreamConfig> {
    // Prefer stereo f32 in 44.1k
    if let Some(config) = configs
        .filter(|c| c.channels() >= 2 && c.sample_format() == cpal::SampleFormat::F32)
        .find(|c| c.min_sample_rate().0 <= SAMPLE_RATE && c.max_sample_rate().0 >= SAMPLE_RATE)
    {
        return Some(config.with_sample_rate(cpal::SampleRate(SAMPLE_RATE)));
    }

    // any f32 in 44.1k
    if let Some(config) = configs
        .filter(|c| c.sample_format() == cpal::SampleFormat::F32)
        .find(|c| c.min_sample_rate().0 <= SAMPLE_RATE && c.max_sample_rate().0 >= SAMPLE_RATE)
    {
        return Some(config.with_sample_rate(cpal::SampleRate(SAMPLE_RATE)));
    }

    // any 44.1k
    if let Some(config) = configs
        .find(|c| c.min_sample_rate().0 <= SAMPLE_RATE && c.max_sample_rate().0 >= SAMPLE_RATE)
    {
        return Some(config.with_sample_rate(cpal::SampleRate(SAMPLE_RATE)));
    }

    None
}
