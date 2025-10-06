use assert_no_alloc::*;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use super::AudioControler;

#[cfg(debug_assertions)]
use assert_no_alloc::AllocDisabler;

pub const SAMPLE_RATE: u32 = 44_100;

#[cfg(debug_assertions)]
#[global_allocator]
static A: AllocDisabler = AllocDisabler;

#[derive(Debug, Clone)]
struct SharedData {
    samples: Arc<Vec<f32>>,
    _sample_rate: u32,
    pos: Arc<AtomicUsize>,
}

fn audio_loop<S>(data: &mut [S], shared: &SharedData)
where
    S: cpal::Sample + cpal::FromSample<f32>,
{
    assert_no_alloc(|| {
        let pos = shared.pos.load(Ordering::Relaxed);
        let buf = &shared.samples;
        let len = buf.len();

        for (i, out_sample) in data.iter_mut().enumerate() {
            let idx = pos + i;
            let sample = if idx < len { buf[idx] } else { 0.0 };

            *out_sample = S::from_sample(sample);
        }

        if pos > shared.samples.len() {
            shared.pos.swap(0, Ordering::Relaxed);
        } else {
            shared.pos.store(pos + data.len(), Ordering::Relaxed);
        }
    });
}

macro_rules! build_stream_match {
    ($device:expr, $shared:expr, $config:expr, $state:expr, $err_fn:expr, { $( $fmt:path => $ty:ty ),* $(,)? }) => {{
        let shared_clone = Arc::clone($shared);
        match $device.default_output_config().unwrap().sample_format() {
            $(
                $fmt => $device.build_output_stream(
                    $config,
                    move |data: &mut [$ty], _| audio_loop(data, &shared_clone),
                    $err_fn,
                    None,
                ),
            )*
            other => panic!("Unsupported sample format {:?}", other),
        }
    }};
}

impl AudioControler {
    pub fn new(samples: Vec<f32>, sample_rate: u32) -> Self {
        let host = cpal::default_host();
        let device = host.default_output_device().expect("No output device");
        let mut supported_configs = device
            .supported_output_configs()
            .expect("Error getting configs");

        let config = pick_config(&mut supported_configs);
        if config.is_none() {
            return Self {};
        }

        println!("{config:#?}");

        let shared_data = Arc::new(SharedData {
            samples: Arc::from(samples),
            _sample_rate: sample_rate,
            pos: Arc::new(AtomicUsize::new(0)),
        });

        let stream = build_stream_match!(
            device,
            &shared_data,
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

        Self {}
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
