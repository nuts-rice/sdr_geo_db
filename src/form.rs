/// Shared form data structures for creating log entries
/// This module is WASM-compatible and can be used by both native TUI and web UI
use crate::model::model::SignalMode;
use serde::{Deserialize, Serialize};

/// Form data for creating a new log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogFormData {
    pub frequency: f32,
    pub latitude: f32,
    pub longitude: f32,
    pub callsign: String,
    pub mode: SignalMode,
    pub comment: String,
    pub recording_duration: f32,
}

impl LogFormData {
    pub fn new() -> Self {
        Self {
            frequency: 0.0,
            latitude: 0.0,
            longitude: 0.0,
            callsign: String::new(),
            mode: SignalMode::FM,
            comment: String::new(),
            recording_duration: 0.0,
        }
    }

    /// Validate the form data
    pub fn validate(&self) -> Result<(), String> {
        if self.frequency <= 0.0 {
            return Err("Frequency must be positive".to_string());
        }

        if self.latitude.abs() > 90.0 {
            return Err("Latitude must be between -90 and 90".to_string());
        }

        if self.longitude.abs() > 180.0 {
            return Err("Longitude must be between -180 and 180".to_string());
        }

        if self.recording_duration < 0.0 {
            return Err("Recording duration must be non-negative".to_string());
        }

        Ok(())
    }
}

impl Default for LogFormData {
    fn default() -> Self {
        Self::new()
    }
}
