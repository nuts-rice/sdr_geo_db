pub mod hackrf;
pub mod stream;

mod file;

const CHUNK_SIZE: usize = 8192;
const MAX_CHUNKS: usize = 1000;

pub trait Source {}

/*pub struct Source {
    device: Device,
    rx: mpsc::Receiver<Vec<u8>>,
    antenna: Option<String>,
    gps_coord: bool,
}

impl Source {
    pub fn new(device: Device, freq: f32, samp_rate: u32) -> Self {
        let (tx, rx) = mpsc::channel::<Vec<u8>>(MAX_CHUNKS);
        Source {
            device,
            rx,
            antenna: None,
            gps_coord: false,
        }
    }
}
*/
