pub const WINDOW_SIZE: isize = 24;

fn sinc(x: f32) -> f32 {
    if x == 0.0 {
        1.0
    } else {
        let pi_x = std::f32::consts::PI * x;
        (pi_x).sin() / pi_x
    }
}

pub fn interpolate(samples: &[f32], pos: f64) -> f32 {
    let len = samples.len() as isize;
    let idx = pos.floor() as isize;
    let frac = (pos - idx as f64) as f32;

    let mut result = 0.0;
    let mut norm = 0.0;

    for i in -WINDOW_SIZE..=WINDOW_SIZE {
        let sample_idx = (idx + i).rem_euclid(len) as usize;
        let weight = sinc(i as f32 - frac);
        result += samples[sample_idx] * weight;
        norm += weight;
    }

    result / norm.max(1e-6)
}
