use super::node::{AudioNode, Param};

#[derive(Debug)]
pub struct GainNode {
    gain: f32,
}

impl Default for GainNode {
    fn default() -> Self {
        Self { gain: 1.0 }
    }
}

impl AudioNode for GainNode {
    fn param_names(&self) -> &'static [&'static str] {
        &["Gain"]
    }

    fn process(&mut self, buffer: &mut [f32]) {
        for sample in buffer.iter_mut() {
            *sample *= self.gain;
        }
    }

    fn set_param(&mut self, idx: usize, value: f32, _sample_rate: f32) {
        if idx == 0 {
            self.gain = value
        }
    }
}
