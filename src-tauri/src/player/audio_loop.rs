use std::sync::{atomic::Ordering, Arc};

use arc_swap::ArcSwap;
use assert_no_alloc::*;

use crate::player::{bus::Bus, SharedAudioBuffer};

#[cfg(debug_assertions)]
#[global_allocator]
static A: AllocDisabler = AllocDisabler;

pub fn audio_loop<S>(data: &mut [S], shared: &Arc<ArcSwap<SharedAudioBuffer>>, bus: &Arc<Bus>)
where
    S: cpal::Sample + cpal::FromSample<f32>,
{
    // TODO: move to alloc free sample loading
    let shared = shared.load();

    assert_no_alloc(|| {
        let pos = shared.pos.load(Ordering::Relaxed);
        let buf = &shared.samples;
        let len = buf.len();

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

#[macro_pub::macro_pub(super)]
macro_rules! build_stream_match {
    ($device:expr, $shared:expr, $bus: expr, $config:expr, $state:expr, $err_fn:expr, { $( $fmt:path => $ty:ty ),* $(,)? }) => {{
        use crate::player::audio_loop::audio_loop;

        let bus_clone = Arc::clone($bus);

        match $device.default_output_config().unwrap().sample_format() {
            $(
                $fmt => $device.build_output_stream(
                    $config,
                    move |data: &mut [$ty], _| audio_loop(data, $shared, &bus_clone),
                    $err_fn,
                    None,
                ),
            )*
            other => panic!("Unsupported sample format {:?}", other),
        }
    }};
}
