use arc_swap::ArcSwap;
use assert_no_alloc::*;
use crossbeam_channel::Receiver;
use std::sync::{Arc, atomic::Ordering};

use super::bus::Bus;
use super::resample::interpolate;
use crate::player::{AudioEvent, PlayerProps, SharedAudioBuffer};

#[cfg(debug_assertions)]
#[global_allocator]
static A: AllocDisabler = AllocDisabler;

pub fn audio_loop<S>(data: &mut [S], state: AudioLoopState, sample_rate: u32)
where
    S: cpal::Sample + cpal::FromSample<f32>,
{
    let bus = state.bus;
    let shared = state.shared;
    let volume = state.props.volume.load(Ordering::Relaxed);

    // TODO: move to alloc free sample loading
    let shared = shared.load();

    let speed = state.props.playback_speed.load(Ordering::Relaxed);
    let ratio = (shared.sample_rate as f64 / sample_rate as f64) * speed as f64;

    assert_no_alloc(|| {
        let mut pos = state.props.position.load(Ordering::Relaxed);
        let len = shared.samples.len() as f64;

        if !state.props.is_playing.load(Ordering::Relaxed) {
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
                pos %= len;
            }
        }

        state.props.position.store(pos, Ordering::Relaxed);
    });
}

#[derive(Clone)]
pub(super) struct AudioLoopState {
    pub _rx: Arc<Receiver<AudioEvent>>,
    pub bus: Arc<Bus>,
    pub shared: Arc<ArcSwap<SharedAudioBuffer>>,
    pub props: Arc<PlayerProps>,
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
                    move |data: &mut [$ty], _| audio_loop(data, $props, sample_rate.0),
                    $err_fn,
                    None,
                ),
            )*
            other => panic!("Unsupported sample format {:?}", other),
        }
    }};
}
