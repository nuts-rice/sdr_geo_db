use std::fmt;

/// Validation errors for SDR geospatial data
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationError {
    InvalidLatitude(f64),
    InvalidLongitude(f64),
    InvalidFrequency(f64),
    InvalidTimestamp(String),
    InvalidBoundingBox(String),
    InvalidRecordingDuration(f32), 
    EmptyDataset,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DatabaseError {
    ConnectionError(String),
    QueryError(String),
    NotFound(String),
    DuplicateEntry(String),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidLatitude(lat) => {
                write!(f, "Invalid latitude: {} (must be between -90 and 90)", lat)
            }
            ValidationError::InvalidLongitude(lon) => {
                write!(
                    f,
                    "Invalid longitude: {} (must be between -180 and 180)",
                    lon
                )
            }
            ValidationError::InvalidFrequency(freq) => {
                write!(f, "Invalid frequency: {} (must be positive)", freq)
            }
            ValidationError::InvalidTimestamp(msg) => {
                write!(f, "Invalid timestamp: {}", msg)
            }
            ValidationError::InvalidBoundingBox(msg) => {
                write!(f, "Invalid bounding box: {}", msg)
            }
            ValidationError::EmptyDataset => {
                write!(f, "Dataset is empty")
            }
            ValidationError::InvalidRecordingDuration(duration) => {
                write!(f, "Invalid recording duration: {} (must be positive)", duration)
            }
        }
    }
}

impl std::error::Error for ValidationError {}
