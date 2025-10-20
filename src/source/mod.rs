pub mod hackrf;
pub mod stream;
use soapysdr::Device;
use tokio::sync::mpsc;

pub struct Source {
    device: Device,
    rx: mpsc::Receiver<Vec<u8>>,
}

impl Source {
    pub fn new(
        freq: f32,
        samp_rate: u32,
    ) -> Result<(Self, Box<mpsc::Sender<Vec<u8>>>), soapysdr::Error> {
        todo!()
    }
}
