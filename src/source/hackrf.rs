use num_complex::Complex;
use soapysdr::{Device, Direction, RxStream};
use tokio::sync::mpsc;

use crate::source::{IQSource, SourceError};

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

#[allow(dead_code)]
pub struct HackRFSource {
    device: Device,
    config: HackRFConfig,
    buffer: Vec<Complex<f32>>,
    rx_stream: Option<RxStream<Complex<f32>>>,
    spectrum_tx: mpsc::Sender<Vec<(f64, f64)>>,
    spectrum_rx: mpsc::Receiver<Vec<(f64, f64)>>,
    is_streaming: bool,
}

impl IQSource for HackRFSource {
    fn read_samples(&mut self, buffer: &mut [Complex<f32>]) -> Result<usize, SourceError> {
        if !self.is_streaming {
            self.device
                .set_sample_rate(Direction::Rx, 0, self.config.sample_rate as f64)
                .map_err(|e| SourceError::DeviceError(e.to_string()))?;
            self.device
                .set_bandwidth(Direction::Rx, 0, self.config.bandwidth as f64)
                .map_err(|e| SourceError::DeviceError(e.to_string()))?;
            self.set_frequency(self.config.center_frequency)?;
            self.set_lna_gain(self.config.lna_gain)?;
            self.set_vga_gain(self.config.vga_gain)?;

            let mut rx_stream = self
                .device
                .rx_stream::<Complex<f32>>(&[0])
                .map_err(|e| SourceError::DeviceError(e.to_string()))?;
            rx_stream
                .activate(None)
                .map_err(|e| SourceError::DeviceError(e.to_string()))?;
            self.rx_stream = Some(rx_stream);
            self.is_streaming = true;
        }

        let rx_stream = self.rx_stream.as_mut().ok_or(SourceError::NotStreaming)?;
        let num_samples = rx_stream
            .read(&mut [buffer], 1000000)
            .map_err(|e| SourceError::DeviceError(e.to_string()))?;

        Ok(num_samples)
    }
    fn set_frequency(&mut self, freq: f32) -> Result<(), SourceError> {
        self.set_frequency(freq)
    }

    fn set_sample_rate(&mut self, rate: f32) -> Result<(), SourceError> {
        self.device
            .set_sample_rate(Direction::Rx, 0, rate as f64)
            .map_err(|e| SourceError::DeviceError(e.to_string()))
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
            spectrum_tx: mpsc::channel(1).0,
            spectrum_rx: mpsc::channel(1).1,
            is_streaming: false,
        })
    }

    pub fn start_streaming(&mut self) {
        let tx = self.spectrum_tx.clone();
        let device = self.device.clone();
        tokio::spawn(async move {
            let mut buffer = vec![Complex::default(); BUFFER_SIZE];
            loop {
                let mut rx_stream = device
                    .rx_stream::<Complex<f32>>(&[0])
                    .expect("Failed to create RX stream");
                rx_stream
                    .activate(None)
                    .expect("Failed to activate RX stream");
                let num_samples = rx_stream
                    .read(&mut [buffer.as_mut_slice()], 1000000)
                    .expect("Failed to read samples");
                //TODO: FFT and compute spectrum

                let spectrum: Vec<(f64, f64)> = vec![];
                if tx.send(spectrum).await.is_err() {
                    break;
                }
            }
        });
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
