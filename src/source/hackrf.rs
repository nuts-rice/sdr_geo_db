use num_complex::Complex;
use soapysdr::{Device, Direction};
use tokio::sync::mpsc;

use crate::source::{Source, SourceError};

pub struct HackRFConfig {
    pub device_index: usize,
    pub center_frequency: f32,
    pub bandwidth: f32,
    pub sample_rate: f32,
    pub lna_gain: usize,
    pub vga_gain: usize,
}

impl Default for HackRFConfig {
    fn default() -> Self {
        HackRFConfig {
            device_index: 0,
            center_frequency: 100e6,
            bandwidth: 20e6,
            sample_rate: 20e6,
            lna_gain: 16,
            vga_gain: 20,
        }
    }
}

pub struct HackRFSource {
    device: Device,
    _config: HackRFConfig,
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

impl HackRFSource {
    pub fn new(config: HackRFConfig) -> Result<Self, SourceError> {
        let device =
            Device::new("driver=hackrf").map_err(|e| SourceError::DeviceError(e.to_string()))?;
        Ok(Self {
            device,
            _config: config,
        })
    }

    pub fn set_frequency(&mut self, frequency: f32) -> Result<(), SourceError> {
        self.device
            .set_frequency(Direction::Rx, 0, frequency as f64, ())
            .map_err(|e| SourceError::DeviceError(e.to_string()))
    }

    // TODO: figure gain channels?
    pub fn set_vga_gain(&mut self, gain: usize) -> Result<(), SourceError> {
        self.device
            .set_gain(Direction::Rx, 1, gain as f64)
            .map_err(|e| SourceError::DeviceError(e.to_string()))
    }
    pub fn set_lna_gain(&mut self, gain: usize) -> Result<(), SourceError> {
        self.device
            .set_gain(Direction::Rx, 2, gain as f64)
            .map_err(|e| SourceError::DeviceError(e.to_string()))
    }
}
