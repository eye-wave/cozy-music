use std::fs::File;
use std::path::Path;
use std::sync::Arc;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::{MediaSourceStream, MediaSourceStreamOptions};
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::{Hint, ProbeResult};
use symphonia::default::{get_codecs, get_probe};

use crate::player::device::SAMPLE_RATE;

use super::{DecoderResult, DecodingError, DecodingResult};

fn create_probe<P: AsRef<Path>>(path: &P) -> Result<ProbeResult, DecodingError> {
    let probe = get_probe();
    let file = File::open(path).unwrap();
    let media_source = MediaSourceStream::new(Box::new(file), MediaSourceStreamOptions::default());

    let probe_result = probe.format(
        &Hint::new(),
        media_source,
        &FormatOptions::default(),
        &MetadataOptions::default(),
    )?;

    Ok(probe_result)
}

pub fn decode_audio<P: AsRef<Path>>(path: &P) -> DecodingResult {
    let mut probe = create_probe(path)?;
    let track = probe.format.default_track().ok_or(DecodingError::NoTrack)?;
    let codec_registry = get_codecs();

    let mut decoder = codec_registry
        .make(&track.codec_params, &DecoderOptions::default())
        .unwrap();

    let sample_rate = track.codec_params.sample_rate.unwrap_or(SAMPLE_RATE);
    let mut channels_data: Vec<Vec<f32>> = Vec::new();
    let mut sample_buf: Option<SampleBuffer<f32>> = None;

    while let Ok(packet) = probe.format.next_packet() {
        let audio_buf = decoder.decode(&packet)?;
        let spec = *audio_buf.spec();
        let duration = audio_buf.capacity() as u64;
        let ch_count = spec.channels.count();

        if channels_data.is_empty() {
            channels_data = vec![Vec::with_capacity(1_000_000); ch_count];
        }

        if sample_buf.is_none() || sample_buf.as_ref().unwrap().capacity() < duration as usize {
            sample_buf = Some(SampleBuffer::<f32>::new(duration, spec));
        }

        let buf = sample_buf.as_mut().unwrap();
        buf.copy_interleaved_ref(audio_buf);

        for frame in buf.samples().chunks_exact(ch_count) {
            for (ch, &s) in frame.iter().enumerate() {
                channels_data[ch].push(s);
            }
        }
    }

    Ok(DecoderResult {
        sample_rate,
        channels: Arc::new(channels_data),
    })
}
