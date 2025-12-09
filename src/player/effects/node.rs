pub trait AudioNode {
    fn param_names(&self) -> &'static [&'static str];

    fn process(&mut self, buffer: &mut [f32]);
    fn set_param(&mut self, idx: usize, value: f32, _sample_rate: f32);
}

pub trait Param {
    fn normalize(&self) -> f32;
    fn denormalize(norm: f32) -> Self;
}
