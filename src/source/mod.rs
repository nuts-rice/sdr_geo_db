use tokio::sync::mpsc;

pub mod hackrf;
pub mod stream;

pub mod file;
pub mod spectrum;

const CHUNK_SIZE: usize = 8192;
const MAX_CHUNKS: usize = 1000;

pub enum SourceError {
    StartError(String),
    StopError(String),
    DeviceError(String),
    StreamError(String),
}


pub trait Source {
    fn start(&mut self) -> Result<(), SourceError>;
    fn stop(&mut self) -> Result<(), SourceError>;
    fn get_receiver(&mut self) -> &mut mpsc::Receiver<Vec<u8>>;
    fn get_device_info(&self) -> String;


}

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
