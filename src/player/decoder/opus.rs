use std::fs::File;
use std::path::Path;

use super::{DecoderResult, DecodingError, DecodingResult};

pub fn decode_audio<P: AsRef<Path>>(path: &P) -> DecodingResult {
    let file = File::open(path)?;
    let (raw, info) = ogg_opus::decode::<_, 48_000>(file)?;

    let channels = info.channels as usize;
    if channels == 0 {
        return Err(DecodingError::NoTrack);
    }

    let mut channels_data: Vec<Vec<f32>> = vec![Vec::new(); channels];

    for frame in raw.chunks_exact(channels) {
        for (ch, &s) in frame.iter().enumerate() {
            channels_data[ch].push(s as f32 / i16::MAX as f32);
        }
    }

    Ok(DecoderResult {
        sample_rate: 48_000,
        channels: channels_data,
    })
}
