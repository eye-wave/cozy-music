use super::node::{AudioNode, Param};

#[derive(Debug)]
pub struct BiquadFilterNode {
    f_type: FilterType,
    q: Q,
    freq: Frequency,
    gain: Gain,

    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,

    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

#[derive(Debug)]
struct Frequency(f32);

#[derive(Debug)]
struct Q(f32);

#[derive(Debug)]
struct Gain(f32);

#[derive(Debug)]
pub enum FilterType {
    Lowpass,
    Highpass,
    Bandpass,
    Allpass,
    Notch,
    Bell,
    Highshelf,
    Lowshelf,
}

impl Param for FilterType {
    fn normalize(&self) -> f32 {
        match self {
            Self::Lowpass => 0.0,
            Self::Highpass => 1.0 / 7.0,
            Self::Bandpass => 2.0 / 7.0,
            Self::Allpass => 3.0 / 7.0,
            Self::Notch => 4.0 / 7.0,
            Self::Bell => 5.0 / 7.0,
            Self::Highshelf => 6.0 / 7.0,
            Self::Lowshelf => 1.0,
        }
    }

    fn denormalize(norm: f32) -> Self {
        if norm < 1.0 / 7.0 {
            Self::Lowpass
        } else if norm < 2.0 / 7.0 {
            Self::Highpass
        } else if norm < 3.0 / 7.0 {
            Self::Bandpass
        } else if norm < 4.0 / 7.0 {
            Self::Allpass
        } else if norm < 5.0 / 7.0 {
            Self::Notch
        } else if norm < 6.0 / 7.0 {
            Self::Bell
        } else if norm < 1.0 {
            Self::Highshelf
        } else {
            Self::Lowshelf
        }
    }
}

impl Param for Frequency {
    fn normalize(&self) -> f32 {
        let f_min = 20.0_f32;
        let f_max = 20_000.0_f32;
        (self.0.log10() - f_min.log10()) / (f_max.log10() - f_min.log10())
    }

    fn denormalize(norm: f32) -> Self {
        let f_min = 20.0_f32;
        let f_max = 20_000.0_f32;
        let freq = 10f32.powf(norm * (f_max.log10() - f_min.log10()) + f_min.log10());
        Self(freq)
    }
}

impl Param for Gain {
    fn normalize(&self) -> f32 {
        let min = -10.0_f32;
        let max = 10.0_f32;

        let sign = self.0.signum();
        let abs = self.0.abs();

        let mag_norm = (abs.exp() - min.abs().exp()) / (max.abs().exp() - min.abs().exp());

        if sign < 0.0 {
            0.5 * (1.0 - mag_norm)
        } else {
            0.5 + 0.5 * mag_norm
        }
    }

    fn denormalize(norm: f32) -> Self {
        let min = -10.0_f32;
        let max = 10.0_f32;

        if norm < 0.5 {
            let mag_norm = 1.0 - (norm / 0.5);
            let val = -((mag_norm * (max.abs().exp() - min.abs().exp()) + min.abs().exp()).ln());
            Self(val)
        } else {
            let mag_norm = (norm - 0.5) / 0.5;
            let val = (mag_norm * (max.abs().exp() - min.abs().exp()) + min.abs().exp()).ln();
            Self(val)
        }
    }
}

impl Param for Q {
    fn normalize(&self) -> f32 {
        let min = -10.0_f32;
        let max = 10.0_f32;
        (self.0 - min) / (max - min)
    }

    fn denormalize(norm: f32) -> Self {
        let min = -10.0_f32;
        let max = 10.0_f32;
        Self(norm * (max - min) + min)
    }
}

impl BiquadFilterNode {
    fn update_coefficients(&mut self, sample_rate: f32) {
        let f = self.freq.0;
        let q = self.q.0.max(0.0001);
        let gain = 10f32.powf(self.gain.0 / 20.0);

        let w0 = 2.0 * std::f32::consts::PI * f / sample_rate;
        let cos_w0 = w0.cos();
        let sin_w0 = w0.sin();

        let alpha = match self.f_type {
            FilterType::Lowpass
            | FilterType::Highpass
            | FilterType::Bandpass
            | FilterType::Allpass
            | FilterType::Notch
            | FilterType::Bell
            | FilterType::Highshelf
            | FilterType::Lowshelf => sin_w0 / (2.0 * q),
        };

        let (b0, b1, b2, a0, a1, a2) = match self.f_type {
            FilterType::Lowpass => {
                let b0 = (1.0 - cos_w0) / 2.0;
                let b1 = 1.0 - cos_w0;
                let b2 = (1.0 - cos_w0) / 2.0;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_w0;
                let a2 = 1.0 - alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            FilterType::Highpass => {
                let b0 = (1.0 + cos_w0) / 2.0;
                let b1 = -(1.0 + cos_w0);
                let b2 = (1.0 + cos_w0) / 2.0;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_w0;
                let a2 = 1.0 - alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            FilterType::Bandpass => {
                let b0 = alpha;
                let b1 = 0.0;
                let b2 = -alpha;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_w0;
                let a2 = 1.0 - alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            FilterType::Allpass => {
                let b0 = 1.0 - alpha;
                let b1 = -2.0 * cos_w0;
                let b2 = 1.0 + alpha;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_w0;
                let a2 = 1.0 - alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            FilterType::Notch => {
                let b0 = 1.0;
                let b1 = -2.0 * cos_w0;
                let b2 = 1.0;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_w0;
                let a2 = 1.0 - alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            FilterType::Bell => {
                let a = alpha * gain;
                let b0 = 1.0 + a;
                let b1 = -2.0 * cos_w0;
                let b2 = 1.0 - a;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_w0;
                let a2 = 1.0 - alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            FilterType::Highshelf => {
                let b0 = gain * ((1.0 + cos_w0) / 2.0);
                let b1 = -gain * (1.0 + cos_w0);
                let b2 = gain * ((1.0 + cos_w0) / 2.0);
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_w0;
                let a2 = 1.0 - alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            FilterType::Lowshelf => {
                let b0 = gain * ((1.0 - cos_w0) / 2.0);
                let b1 = gain * (1.0 - cos_w0);
                let b2 = gain * ((1.0 - cos_w0) / 2.0);
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_w0;
                let a2 = 1.0 - alpha;
                (b0, b1, b2, a0, a1, a2)
            }
        };

        self.b0 = b0 / a0;
        self.b1 = b1 / a0;
        self.b2 = b2 / a0;
        self.a1 = a1 / a0;
        self.a2 = a2 / a0;
    }
}

impl AudioNode for BiquadFilterNode {
    fn param_names(&self) -> &'static [&'static str] {
        &["Type", "Freq", "Q", "Gain"]
    }

    fn process(&mut self, buffer: &mut [f32]) {
        for sample in buffer.iter_mut() {
            let x0 = *sample;
            let y0 = self.b0 * x0 + self.b1 * self.x1 + self.b2 * self.x2
                - self.a1 * self.y1
                - self.a2 * self.y2;

            self.x2 = self.x1;
            self.x1 = x0;

            self.y2 = self.y1;
            self.y1 = y0;

            *sample = y0;
        }
    }

    fn set_param(&mut self, idx: usize, value: f32, sample_rate: f32) {
        match idx {
            0 => {
                let f_type = FilterType::denormalize(value);

                self.f_type = f_type;
                self.update_coefficients(sample_rate);
            }
            1 => self.freq = Frequency::denormalize(value),
            2 => self.q = Q::denormalize(value),
            3 => self.gain = Gain::denormalize(value),
            _ => {}
        }
    }
}
