use crate::error::ValidationError;
use chrono::{DateTime, NaiveDateTime, Utc};
use diesel::prelude::*;

pub enum SignalMode {
    FM,
    AM,
    USB,
    LSB,
    CW,
}

/// Database representation of an SDR measurement log entry
#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::logs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Log {
    pub id: i32,
    pub frequency: f32,
    pub xcoord: f32,
    pub ycoord: f32,
    pub callsign: Option<String>,
    pub mode: String,
    pub comment: Option<String>,
    pub timestamp: NaiveDateTime,
    pub recording_duration: f32,
}

/// New log entry for insertion into database
#[derive(Insertable)]
#[diesel(table_name = crate::schema::logs)]
pub struct NewLog<'a> {
    pub frequency: f32,
    pub xcoord: f32,
    pub ycoord: f32,
    pub callsign: &'a str,
    pub mode: &'a str,
    pub comment: Option<&'a str>,
    pub recording_duration: f32,
    // timestamp will use database default (CURRENT_TIMESTAMP)
}

impl Log {
    /// Get timestamp as DateTime<Utc>
    pub fn timestamp_utc(&self) -> DateTime<Utc> {
        DateTime::<Utc>::from_naive_utc_and_offset(self.timestamp, Utc)
    }

    /// Get frequency in Hz (converts from MHz stored in DB)
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
        xcoord: f32,
        ycoord: f32,
        callsign: &'a str,
        mode: &'a str,
        comment: Option<&'a str>,
        recording_duration: f32,
    ) -> Result<Self, ValidationError> {
        // Validate frequency must be positive
        if frequency <= 0.0 {
            return Err(ValidationError::InvalidFrequency(frequency as f64));
        }
        if xcoord < -180.0 || xcoord > 180. {
            return Err(ValidationError::InvalidLatitude(xcoord as f64));
        }
        if ycoord < -90. || ycoord > 90. {
            return Err(ValidationError::InvalidLongitude(ycoord as f64));
        }
        if recording_duration < 0. {
            return Err(ValidationError::InvalidRecordingDuration(recording_duration));
        }

        Ok(NewLog {
            frequency,
            xcoord,
            ycoord,
            callsign,
            mode,
            comment,
            recording_duration,
        })
    }
}

/// Render a log entry to the console
pub fn render(log: &Log) {
    println!(
        "{} MHz | Callsign: {} | Coordinate: ({}, {}) | Comment: {:?} | Mode: {} | Recorded at: {} | Duration: {:.2} sec",
        log.frequency / 1_000_000.,
        log.callsign.as_deref().unwrap_or("").to_uppercase(),
        log.xcoord,
        log.ycoord,
        log.comment.as_deref().unwrap_or(""),
        log.mode,
        log.timestamp,
        log.recording_duration,
        
    );
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
