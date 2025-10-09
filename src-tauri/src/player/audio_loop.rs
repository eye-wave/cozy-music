use arc_swap::ArcSwap;
use assert_no_alloc::*;
use crossbeam_channel::Receiver;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use crate::player::{bus::Bus, AudioEvent, SharedAudioBuffer};

#[cfg(debug_assertions)]
#[global_allocator]
static A: AllocDisabler = AllocDisabler;

pub fn audio_loop<S>(data: &mut [S], props: AudioLoopProps)
where
    S: cpal::Sample + cpal::FromSample<f32>,
{
    let bus = props.bus;
    let shared = props.shared;
    let is_playing = props.is_playing;

    // TODO: move to alloc free sample loading
    let shared = shared.load();

    assert_no_alloc(|| {
        let pos = shared.pos.load(Ordering::Relaxed);
        let buf = &shared.samples;
        let len = buf.len();

        if is_playing.load(Ordering::Relaxed) {
            data.fill(cpal::Sample::EQUILIBRIUM);
            return;
        }

        for (i, out_sample) in data.iter_mut().enumerate() {
            let idx = pos + i;
            let sample = if idx < len { buf[idx] } else { 0.0 };

            bus.send(sample);
            *out_sample = S::from_sample(sample);
        }

        if pos > shared.samples.len() {
            shared.pos.swap(0, Ordering::Relaxed);
        } else {
            shared.pos.store(pos + data.len(), Ordering::Relaxed);
        }
    });
}

#[derive(Clone)]
pub(super) struct AudioLoopProps {
    pub _rx: Arc<Receiver<AudioEvent>>,
    pub bus: Arc<Bus>,
    pub shared: Arc<ArcSwap<SharedAudioBuffer>>,
    pub is_playing: Arc<AtomicBool>,
}

#[macro_pub::macro_pub(super)]
macro_rules! build_stream_match {
    ($device:expr, $props: expr, $config:expr, $state:expr, $err_fn:expr, { $( $fmt:path => $ty:ty ),* $(,)? }) => {{
        use crate::player::audio_loop::audio_loop;

        match $device.default_output_config().unwrap().sample_format() {
            $(
                $fmt => $device.build_output_stream(
                    $config,
                    move |data: &mut [$ty], _| audio_loop(data, $props),
                    $err_fn,
                    None,
                ),
            )*
            other => panic!("Unsupported sample format {:?}", other),
        }
    }};
}
