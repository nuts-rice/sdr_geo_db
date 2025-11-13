use soapysdr::RxStream;
use tokio::sync::mpsc;

use crate::source::{SourceError, Source};

pub struct HackRFSource {
    rx_stream: RxStream<f32>,
}

impl Source for HackRFSource {
    fn start(&mut self) -> Result<(), SourceError> {
        Ok(())
    }
    
    fn stop(&mut self) -> Result<(), SourceError> {
        Ok(())
    }
    fn get_receiver(&mut self) -> &mut mpsc::Receiver<Vec<u8>> {
        todo!()
    }
    fn get_device_info(&self) -> String {
        todo!()
    }
    
}

impl HackRFSource {}
