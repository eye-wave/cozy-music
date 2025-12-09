pub trait AudioNode {
    const PARAM_NAMES: &'static [&'static  str];

    fn process(&mut self, buffer: &mut [f32]) {}
}
