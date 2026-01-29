use num_complex::Complex;

use std::fmt;

pub mod fft;
pub mod hackrf;
pub mod stream;

pub mod cache;
pub mod file;
pub mod sdr;
pub mod spectrum;
#[derive(Debug)]
pub enum SourceError {
    StartError(String),
    StopError(String),
    DeviceError(String),
    IOError(std::io::Error),
    StreamError(String),
    NotStreaming,
}

impl fmt::Display for SourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SourceError::StartError(msg) => write!(f, "Source Start Error: {}", msg),
            SourceError::StopError(msg) => write!(f, "Source Stop Error: {}", msg),
            SourceError::DeviceError(msg) => write!(f, "Source Device Error: {}", msg),
            SourceError::IOError(err) => write!(f, "Source IO Error: {}", err),
            SourceError::StreamError(msg) => write!(f, "Source Stream Error: {}", msg),
            SourceError::NotStreaming => write!(f, "Source Not Streaming Error"),
        }
    }
}

//Streaming IQ data
#[allow(dead_code)]
trait IQSource: Send {
    fn read_samples(&mut self, buffer: &mut [Complex<f32>]) -> Result<usize, SourceError>;
    fn set_frequency(&mut self, freq: f32) -> Result<(), SourceError>;
    fn set_sample_rate(&mut self, rate: f32) -> Result<(), SourceError>;
}
