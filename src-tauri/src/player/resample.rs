pub const WINDOW_SIZE: isize = 24;

fn sinc(x: f32) -> f32 {
    if x.abs() < 1e-8 {
        1.0
    } else {
        let px = std::f32::consts::PI * x;
        px.sin() / px
    }
}

fn hann_window(x: f32, half_width: f32) -> f32 {
    0.5 * (1.0 + (std::f32::consts::PI * x / half_width).cos())
}

pub fn interpolate(samples: &[f32], pos: f64) -> f32 {
    let len = samples.len() as isize;
    let idx = pos.floor() as isize;
    let frac = (pos - idx as f64) as f32;

    let mut acc = 0.0;
    let mut norm = 0.0;

    for i in -WINDOW_SIZE..=WINDOW_SIZE {
        let sample_idx = (idx + i).clamp(0, len - 1) as usize;
        let x = i as f32 - frac;
        let weight = sinc(x) * hann_window(x, WINDOW_SIZE as f32);
        acc += samples[sample_idx] * weight;
        norm += weight;
    }

    acc / norm.max(1e-6)
}
