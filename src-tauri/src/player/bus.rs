use crossbeam_channel::{Receiver, Sender, bounded};

pub const BUFFER_SIZE: usize = 2048;

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct Bus {
    tx: Sender<f32>,
    rx: Receiver<f32>,
}

impl Default for Bus {
    fn default() -> Self {
        Self::new(BUFFER_SIZE)
    }
}

#[allow(unused)]
impl Bus {
    pub fn new(size: usize) -> Self {
        let (tx, rx) = bounded(size);
        Self { tx, rx }
    }

    pub fn read(&self) -> Vec<f32> {
        self.rx.try_iter().collect()
    }

    pub fn send(&self, value: f32) {
        let _ = self.tx.try_send(value);
    }
}
