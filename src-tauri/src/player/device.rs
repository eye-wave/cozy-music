use assert_no_alloc::*;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::f32::consts::PI;
use std::thread;
use std::time::Duration;

use super::AudioControler;

#[cfg(debug_assertions)]
use assert_no_alloc::AllocDisabler;

pub const SAMPLE_RATE: u32 = 44_100;
pub const MAX_BUFFER_SIZE: usize = 8192;

#[cfg(debug_assertions)]
#[global_allocator]
static A: AllocDisabler = AllocDisabler;

thread_local! {
    static AUDIO_TICK: std::cell::Cell<usize> = std::cell::Cell::new(0);
}

fn audio_loop<S>(data: &mut [S])
where
    S: cpal::Sample + cpal::FromSample<f32>,
{
    assert_no_alloc(|| {
        let buf = &mut [0.0f32; MAX_BUFFER_SIZE];
        let processed = true;

        AUDIO_TICK.with(|tick_cell| {
            let mut tick = tick_cell.get();
            let phase_inc = 2.0 * PI * 440.0 / SAMPLE_RATE as f32;

            for (i, sample) in buf.iter_mut().enumerate() {
                *sample = ((tick + i) as f32 * phase_inc).sin();
            }

            tick += data.len();
            tick_cell.set(tick);
        });

        for i in 0..data.len() {
            let sample = if processed { buf[i] } else { 0.0 };
            data[i] = S::from_sample(sample);
        }
    });
}

macro_rules! build_stream_match {
    ($device:expr, $config:expr, $state:expr,$err_fn:expr, { $( $fmt:path => $ty:ty ),* $(,)? }) => {
        match $device.default_output_config().unwrap().sample_format() {
            $(
                $fmt => $device.build_output_stream(
                    $config,
                    |data: &mut [$ty], _| audio_loop(data),
                    $err_fn,
                    None,
                ),
            )*
            other => panic!("Unsupported sample format {:?}", other),
        }
    };
}

impl AudioControler {
    pub fn new() -> Self {
        let host = cpal::default_host();
        let device = host.default_output_device().expect("No output device");
        let mut supported_configs = device
            .supported_output_configs()
            .expect("Error getting configs");

        let config = pick_config(&mut supported_configs);
        if config.is_none() {
            return Self {
                ..Default::default()
            };
        }

        let stream = build_stream_match!(
            device,
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

        Self {
            ..Default::default()
        }
    }
}

fn pick_config(configs: &mut cpal::SupportedOutputConfigs) -> Option<cpal::SupportedStreamConfig> {
    // Best pick 44.1k and f32
    if let Some(config) = configs.find(|c| {
        c.min_sample_rate().0 <= SAMPLE_RATE
            && c.max_sample_rate().0 >= SAMPLE_RATE
            && c.sample_format() == cpal::SampleFormat::F32
    }) {
        return Some(config.with_sample_rate(cpal::SampleRate(SAMPLE_RATE)));
    }

    // at least 44.1k
    if let Some(config) = configs
        .find(|c| c.min_sample_rate().0 <= SAMPLE_RATE && c.max_sample_rate().0 >= SAMPLE_RATE)
    {
        return Some(config.with_sample_rate(cpal::SampleRate(SAMPLE_RATE)));
    }

    None
}
