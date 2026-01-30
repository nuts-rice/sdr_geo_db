use std::str::FromStr;
use tokio::sync::mpsc;

use crate::spectrum_types::SpectrumFrame;

use super::hackrf::{HackRFConfig, HackRFSource};

#[derive(Clone, Debug, Default)]
pub enum SdrDevice {
    HackRF,
    RTLSDR,
    #[default]
    Mock,
}

impl FromStr for SdrDevice {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "hackrf" => Ok(SdrDevice::HackRF),
            "rtlsdr" => Ok(SdrDevice::RTLSDR),
            "mock" => Ok(SdrDevice::Mock),
            _ => Err(format!("Unsupported SDR device: {}", s)),
        }
    }
}

pub struct SdrSource {
    device: SdrDevice,
    rx: mpsc::Receiver<SpectrumFrame>,
    _tx: mpsc::Sender<SpectrumFrame>,
    _hackrf: Option<HackRFSource>,
}

impl SdrSource {
    pub fn new(device: SdrDevice, config: Option<HackRFConfig>) -> Result<Self, String> {
        let (tx, rx) = mpsc::channel::<SpectrumFrame>(16);

        match device {
            SdrDevice::Mock => {
                let tx_clone = tx.clone();
                tokio::spawn(async move {
                    Self::mock_generator(tx_clone).await;
                });
                Ok(Self {
                    device,
                    rx,
                    _tx: tx,
                    _hackrf: None,
                })
            }
            SdrDevice::HackRF => {
                let hackrf_config = config.unwrap_or_default();
                let mut hackrf = HackRFSource::new(hackrf_config)
                    .map_err(|e| format!("Failed to create HackRF source: {}", e))?;

                // Start streaming
                hackrf.start_streaming(tx.clone());

                Ok(Self {
                    device,
                    rx,
                    _tx: tx,
                    _hackrf: Some(hackrf),
                })
            }
            SdrDevice::RTLSDR => Err("RTLSDR not yet implemented".to_string()),
        }
    }

    /// Receive the next spectrum frame (async)
    pub async fn recv(&mut self) -> Option<SpectrumFrame> {
        self.rx.recv().await
    }

    /// Get device type
    pub fn device(&self) -> &SdrDevice {
        &self.device
    }

    async fn mock_generator(tx: mpsc::Sender<SpectrumFrame>) {
        use std::time::Duration;
        use tokio::time::{Instant, interval};

        let mut interval = interval(Duration::from_millis(66)); // 15 FPS
        let start = Instant::now();

        loop {
            interval.tick().await;
            let elapsed = start.elapsed().as_secs_f64();
            let frame = crate::spectrum_types::generate_mock_spectrum(elapsed);
            if tx.send(frame).await.is_err() {
                break; // Receiver dropped
            }
        }
    }

    pub async fn stop(&mut self) {
        if let Some(hackrf) = &mut self._hackrf {
            hackrf.stop_streaming();
        }
    }
}
