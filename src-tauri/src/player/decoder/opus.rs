use std::fs::File;
use std::path::Path;

use super::{DecoderResult, DecodingResult};

pub fn decode_audio<P: AsRef<Path>>(path: &P) -> DecodingResult {
    let file = File::open(path)?;
    let (raw, _) = ogg_opus::decode::<_, 16000>(file)?;

    let samples = raw
        .iter()
        .map(|sample| *sample as f32 / i16::MAX as f32)
        .collect();

    Ok(DecoderResult {
        samples,
        sample_rate: 48_000,
    })
}
