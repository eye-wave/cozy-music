use arc_swap::ArcSwap;
use assert_no_alloc::*;
use atomic_float::AtomicF32;
use crossbeam_channel::Receiver;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use crate::player::{AudioEvent, SharedAudioBuffer, bus::Bus};

#[cfg(debug_assertions)]
#[global_allocator]
static A: AllocDisabler = AllocDisabler;

pub fn audio_loop<S>(data: &mut [S], state: AudioLoopState, sample_rate: f32)
where
    S: cpal::Sample + cpal::FromSample<f32>,
{
    let bus = state.bus;
    let shared = state.shared;
    let volume = state.volume.load(Ordering::Relaxed);
    let speed = state.playback_speed.load(Ordering::Relaxed);

    // TODO: move to alloc free sample loading
    let shared = shared.load();
    let ratio = (shared.sample_rate as f32 / sample_rate) * speed;

    assert_no_alloc(|| {
        let mut pos = state.position.load(Ordering::Relaxed);
        let len = shared.samples.len() as f32;

        if !state.is_playing.load(Ordering::Relaxed) {
            data.fill(cpal::Sample::EQUILIBRIUM);
            return;
        }

        for out_sample in data.iter_mut() {
            let mut sample = interpolate(&shared.samples, pos);
            sample *= volume;

            bus.send(sample);
            *out_sample = S::from_sample(sample);

            pos += ratio;
            if pos >= len {
                pos -= len;
            }
        }

        state.position.store(pos, Ordering::Relaxed);
    });
}

fn interpolate(samples: &[f32], pos: f32) -> f32 {
    let len = samples.len();
    if len == 0 {
        return 0.0;
    }

    let i = pos.floor() as usize;
    let frac = pos - i as f32;

    if i + 1 >= len {
        return samples[len - 1];
    }

    let s0 = samples[i];
    let s1 = samples[i + 1];
    s0 + (s1 - s0) * frac
}

#[derive(Clone)]
pub(super) struct AudioLoopState {
    pub _rx: Arc<Receiver<AudioEvent>>,
    pub bus: Arc<Bus>,
    pub shared: Arc<ArcSwap<SharedAudioBuffer>>,
    pub is_playing: Arc<AtomicBool>,
    pub position: Arc<AtomicF32>,
    pub volume: Arc<AtomicF32>,
    pub playback_speed: Arc<AtomicF32>,
}

#[macro_pub::macro_pub(super)]
macro_rules! build_stream_match {
    ($device:expr, $props: expr, $config:expr, $state:expr, $err_fn:expr, { $( $fmt:path => $ty:ty ),* $(,)? }) => {{
        use crate::player::audio_loop::audio_loop;

        let sample_rate = $config.sample_rate;

        match $device.default_output_config().unwrap().sample_format() {
            $(
                $fmt => $device.build_output_stream(
                    $config,
                    move |data: &mut [$ty], _| audio_loop(data, $props, sample_rate.0 as f32),
                    $err_fn,
                    None,
                ),
            )*
            other => panic!("Unsupported sample format {:?}", other),
        }
    }};
}
