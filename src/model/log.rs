use crate::error::ValidationError;
use chrono::{DateTime, NaiveDateTime, Utc};
#[cfg(feature = "db")]
use diesel::prelude::*;
use serde::Serialize;

//Frequency is in MHz
#[derive(Serialize, serde::Deserialize, Debug, Clone, Copy, Default, PartialEq)]
pub enum SignalMode {
    #[default]
    FM,
    AM,
    USB,
    LSB,
    CW,
}

/// Database representation of an SDR measurement log entry
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "db", derive(Queryable, Selectable))]
#[cfg_attr(feature = "db", diesel(table_name = crate::schema::logs))]
#[cfg_attr(feature = "db", diesel(check_for_backend(diesel::pg::Pg)))]
pub struct Log {
    pub id: i32,
    pub frequency: f32,
    pub grid: Option<String>,
    pub callsign: Option<String>,
    pub mode: String,
    pub comment: Option<String>,
    pub timestamp: NaiveDateTime,
    pub recording_duration: f32,
}

/// New log entry for insertion into database
#[derive(Debug, Clone)]
#[cfg_attr(feature = "db", derive(Insertable))]
#[cfg_attr(feature = "db", diesel(table_name = crate::schema::logs))]
pub struct NewLog<'a> {
    pub frequency: f32,
    pub grid: &'a str,
    pub callsign: &'a str,
    pub mode: &'a str,
    pub comment: &'a str,
    pub recording_duration: f32,
    pub timestamp: NaiveDateTime,
}

impl Log {
    /// Get timestamp as DateTime<Utc>
    pub fn timestamp_utc(&self) -> DateTime<Utc> {
        DateTime::<Utc>::from_naive_utc_and_offset(self.timestamp, Utc)
    }

    pub fn frequency_hz(&self) -> f64 {
        self.frequency as f64
    }
}

impl<'a> NewLog<'a> {
    /// Create a new NewLog with validation
    ///
    /// # Arguments
    /// * `frequency` - Signal frequency in Hz (must be positive)
    /// * `location` - Geographic position (WGS84, as DbPoint)
    /// * `callsign` - Station callsign
    /// * `bandwidth` - Signal bandwidth in Hz
    /// * `mode` - Operating mode (e.g., "FM", "AM", "SSB")
    /// * `power` - Optional signal power in dBm
    /// * `snr` - Optional Signal-to-Noise Ratio in dB
    /// * `comment` - Optional comment
    ///
    /// # Errors
    /// Returns `ValidationError::InvalidFrequency` if frequency is not positive
    pub fn new(
        frequency: f32,
        grid: &'a str,
        callsign: &'a str,
        mode: &'a str,
        comment: &'a str,
        recording_duration: f32,
        timestamp: NaiveDateTime,
    ) -> Result<Self, ValidationError> {
        // Validate frequency must be positive
        if frequency <= 0.0 {
            return Err(ValidationError::InvalidFrequency(frequency as f64));
        }

        // Validate grid square format
        let grid_len = grid.len();
        if grid_len != 4 && grid_len != 6 {
            return Err(ValidationError::InvalidGridSquare(grid.to_string()));
        }

        if recording_duration < 0. {
            return Err(ValidationError::InvalidRecordingDuration(
                recording_duration,
            ));
        }

        Ok(NewLog {
            frequency,
            grid,
            callsign,
            mode,
            comment,
            recording_duration,
            timestamp,
        })
    }
}

/// Render a log entry to the console
pub fn render_log(log: &Log) -> String {
    let log_string = format!(
        "{} MHz | Callsign: {} | Grid: {} \n
        | Comment: {:?} | Mode: {} | Recorded at: {} \n
        | Duration: {:.2} sec",
        log.frequency,
        log.callsign.as_deref().unwrap_or("").to_uppercase(),
        log.grid.as_deref().unwrap_or(""),
        log.comment.as_deref().unwrap_or(""),
        log.mode,
        log.timestamp,
        log.recording_duration,
    );
    log_string
}

pub fn render_new_log(new_log: &NewLog) -> String {
    let log_string = format!(
        "{} MHz | Callsign: {} | Grid: {} \n
        | Comment: {:?} | Mode: {} | Recorded at: {} \n
        | Duration: {:.2} sec",
        new_log.frequency,
        new_log.callsign.to_uppercase(),
        new_log.grid,
        new_log.comment,
        new_log.mode,
        new_log.timestamp,
        new_log.recording_duration,
    );
    log_string
}

impl SignalMode {
    pub fn to_str(&self) -> &str {
        match self {
            SignalMode::FM => "FM",
            SignalMode::AM => "AM",
            SignalMode::USB => "USB",
            SignalMode::LSB => "LSB",
            SignalMode::CW => "CW",
        }
    }
}

// WIP: parsing mode from input
pub fn parse_mode(mode: &str) -> &str {
    match mode.to_lowercase().as_str() {
        "fm" => "FM",
        "am" => "AM",
        "usb" => "USB",
        "lsb" => "LSB",
        "cw" => "CW",
        _ => "UNKNOWN",
    }
}
