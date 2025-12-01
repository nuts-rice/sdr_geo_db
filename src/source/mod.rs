use num_complex::Complex;
use tokio::sync::mpsc;

use std::fmt;

pub mod hackrf;
pub mod stream;

pub mod file;
pub mod spectrum;


#[derive(Debug)]
pub enum SourceError {
    StartError(String),
    StopError(String),
    DeviceError(String),
    IOError(std::io::Error),
    StreamError(String),
}

impl fmt::Display for SourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SourceError::StartError(msg) => write!(f, "Source Start Error: {}", msg),
            SourceError::StopError(msg) => write!(f, "Source Stop Error: {}", msg),
            SourceError::DeviceError(msg) => write!(f, "Source Device Error: {}", msg),
            SourceError::IOError(err) => write!(f, "Source IO Error: {}", err),
            SourceError::StreamError(msg) => write!(f, "Source Stream Error: {}", msg),
        }
    }
}

#[async_trait::async_trait]
pub trait Source: Send {
    async fn start(&mut self) -> Result<(), SourceError>;
    async fn stop(&mut self) -> Result<(), SourceError>;
    async fn next_samples(&mut self) -> Result<Option<Vec<Complex<f32>>>, SourceError>;
    fn get_receiver(&mut self) -> &mut mpsc::Receiver<Vec<u8>>;
    fn get_device_info(&self) -> String;
    fn get_center_frequency(&self) -> f32;
}

