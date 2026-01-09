use crate::Log;
use serde::{Deserialize, Serialize};
#[derive(Deserialize, Serialize)]
pub struct LogResponse {
    id: i32,
    frequency: f32,
    grid: Option<String>,
    callsign: Option<String>,
    mode: String,
    comment: Option<String>,
    timestamp: String,
    recording_duration: f32,
}

impl From<Log> for LogResponse {
    fn from(log: Log) -> Self {
        let timestamp = log.timestamp_utc().to_rfc3339();
        LogResponse {
            id: log.id,
            frequency: log.frequency,
            grid: log.grid,
            callsign: log.callsign,
            mode: log.mode,
            comment: log.comment,
            timestamp,
            recording_duration: log.recording_duration,
        }
    }
}

#[derive(Serialize)]
pub struct LogsResponse {
    pub logs: Vec<LogResponse>,
    pub count: usize,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}
