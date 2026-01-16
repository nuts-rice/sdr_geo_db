use crate::Log;
use serde::{Deserialize, Serialize};
#[derive(Deserialize, Serialize, Debug)]
pub struct LogResponse {
    id: i32,
    pub frequency: f32,
    pub grid: Option<String>,
    pub callsign: Option<String>,
    pub mode: String,
    pub comment: Option<String>,
    pub timestamp: String,
    pub recording_duration: f32,
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

impl LogResponse {
    //Formatting for table widget
    pub fn to_table_row(&self) -> Vec<String> {
        vec![
            self.id.to_string(),
            format!("{:.3}", self.frequency),
            self.grid.clone().unwrap_or_default(),
            self.callsign.clone().unwrap_or_default(),
            self.mode.clone(),
            self.comment.clone().unwrap_or_default(),
            self.timestamp.clone(),
            format!("{:.2}", self.recording_duration),
        ]
    }
}

#[derive(Serialize, Deserialize)]
pub struct LogsResponse {
    pub logs: Vec<LogResponse>,
    pub count: usize,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}
