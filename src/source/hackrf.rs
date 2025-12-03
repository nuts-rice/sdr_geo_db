use num_complex::Complex;
use soapysdr::{Device, Direction, RxStream};
use tokio::sync::mpsc;

use crate::source::{Source, SourceError};

const BUFFER_SIZE: usize = 262144;

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
    config: HackRFConfig,
    buffer: Vec<Complex<f32>>,
    rx_stream: Option<RxStream<Complex<f32>>>,
    receiver: mpsc::Receiver<Vec<u8>>,
    is_streaming: bool,
}

#[async_trait::async_trait]
impl Source for HackRFSource {
    async fn next_samples(&mut self) -> Result<Option<Vec<Complex<f32>>>, SourceError> {
        if let Some(rx_stream) = &mut self.rx_stream {
            let mut buffer = vec![Complex::<f32>::new(0.0, 0.0); BUFFER_SIZE];
            let samples_read = rx_stream
                .read(&mut [buffer.as_mut_slice()], 1000000)
                .map_err(|e| SourceError::DeviceError(e.to_string()))?;
            if samples_read > 0 {
                buffer.truncate(samples_read);
                Ok(Some(buffer))
            } else {
                Ok(None)
            }
        } else {
            Err(SourceError::NotStreaming)
        }
    }
    async fn start(&mut self) -> Result<(), SourceError> {
        self.device
            .set_sample_rate(Direction::Rx, 0, self.config.sample_rate as f64)
            .map_err(|e| SourceError::DeviceError(e.to_string()))?;
        self.device
            .set_bandwidth(Direction::Rx, 0, self.config.bandwidth as f64)
            .map_err(|e| SourceError::DeviceError(e.to_string()))?;
        self.set_frequency(self.config.center_frequency)?;
        self.set_lna_gain(self.config.lna_gain)?;
        self.set_vga_gain(self.config.vga_gain)?;

        let rx_stream = self
            .device
            .rx_stream::<Complex<f32>>(&[0])
            .map_err(|e| SourceError::DeviceError(e.to_string()))?;
        self.rx_stream = Some(rx_stream);
        self.is_streaming = true;
        Ok(())
    }
    async fn stop(&mut self) -> Result<(), SourceError> {
        if self.rx_stream.is_some() {
            self.rx_stream = None;
        }
        self.is_streaming = false;
        Ok(())
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
            config,
            buffer: Vec::with_capacity(BUFFER_SIZE),
            rx_stream: None,
            receiver: mpsc::channel(100).1,
            is_streaming: false,
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
