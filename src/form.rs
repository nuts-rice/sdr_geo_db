/// Shared form data structures for creating log entries
/// This module is WASM-compatible and can be used by both native TUI and web UI
use crate::model::log::SignalMode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogFormData {
    pub frequency: f32,
    pub grid_square: String,
    pub callsign: String,
    pub mode: SignalMode,
    pub comment: String,
    pub recording_duration: f32,
}

impl LogFormData {
    pub fn new() -> Self {
        Self {
            frequency: 0.0,
            grid_square: String::new(),
            callsign: String::new(),
            mode: SignalMode::FM,
            comment: String::new(),
            recording_duration: 0.0,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.frequency <= 0.0 {
            return Err("Frequency must be positive".to_string());
        }

        // Validate grid square format (should be 4 or 6 characters)
        let grid_len = self.grid_square.len();
        if grid_len != 4 && grid_len != 6 {
            return Err("Grid square must be 4 or 6 characters".to_string());
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
