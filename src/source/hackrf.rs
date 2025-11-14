use num_complex::Complex;
use soapysdr::RxStream;
use tokio::sync::mpsc;

use crate::source::{Source, SourceError};

pub struct HackRFSource {
    rx_stream: RxStream<f32>,
}

#[async_trait::async_trait]
impl Source for HackRFSource {
    async fn next_samples(&mut self) -> Result<Option<Vec<Complex<f32>>>, SourceError> {
        todo!()
    }
    async fn start(&mut self) -> Result<(), SourceError> {
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), SourceError> {
        Ok(())
    }
    fn get_receiver(&mut self) -> &mut mpsc::Receiver<Vec<u8>> {
        todo!()
    }
    fn get_device_info(&self) -> String {
        todo!()
    }
    fn get_center_frequency(&self) -> f32 {
        todo!()
    }
}

impl HackRFSource {}
