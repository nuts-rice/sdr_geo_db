use crate::error::ValidationError;
use crate::schema::log;
use chrono::{DateTime, NaiveDateTime, Utc};
use diesel::prelude::*;

/// Database representation of an SDR measurement log entry
#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::log)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Log {
    pub id: i32,
    pub frequency: i32,
    pub xcoord: f32,
    pub ycoord: f32,
    pub callsign: String,
    pub bandwidth: i32,
    pub mode: String,
    pub power: Option<f64>,
    pub snr: Option<f64>,
    pub comment: Option<String>,
    pub timestamp: NaiveDateTime,
}

/// New log entry for insertion into database
#[derive(Insertable)]
#[diesel(table_name = log)]
pub struct NewLog<'a> {
    pub frequency: i32,
    pub xcoord: f32,
    pub ycoord: f32,
    pub callsign: &'a str,
    pub bandwidth: i32,
    pub mode: &'a str,
    pub power: Option<f64>,
    pub snr: Option<f64>,
    pub comment: Option<&'a str>,
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

    /// Get bandwidth in Hz
    pub fn bandwidth_hz(&self) -> f64 {
        self.bandwidth as f64
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
        frequency: i32,
        xcoord: f32,
        ycoord: f32,
        callsign: &'a str,
        bandwidth: i32,
        mode: &'a str,
        power: Option<f64>,
        snr: Option<f64>,
        comment: Option<&'a str>,
    ) -> Result<Self, ValidationError> {
        // Validate frequency must be positive
        if frequency <= 0 {
            return Err(ValidationError::InvalidFrequency(frequency as f64));
        }

        Ok(NewLog {
            frequency,
            xcoord,
            ycoord,
            callsign,
            bandwidth,
            mode,
            power,
            snr,
            comment,
        })
    }
}

/// Render a log entry to the console
pub fn render(log: &Log) {
    println!(
        "{} MHz | ({}, {}) | {:?} | {} | {} | {}",
        log.frequency / 1_000_000,
        log.callsign.to_uppercase(),
        log.xcoord,
        log.ycoord,
        log.comment.as_deref().unwrap_or(""),
        log.mode,
        log.timestamp,
    );
}
