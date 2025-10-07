use sdr_db::spatial::Coordinate;
use sdr_db::Measurement;
use sdr_db::error::ValidationError;
use chrono::Utc;

#[test]
fn test_measurement_new_valid() {
    // Test case from contracts/api-contract.md lines 42-53
    let m = Measurement::new(
        Coordinate::new(37.7749, -122.4194).unwrap(),
        Utc::now(),
        2.45e9,
        -65.5,
        20e6,
        18.3,
    );
    assert!(m.is_ok());
}

#[test]
fn test_measurement_new_invalid_frequency() {
    // Test case from contracts/api-contract.md lines 56-66
    let m = Measurement::new(
        Coordinate::new(37.7749, -122.4194).unwrap(),
        Utc::now(),
        -100.0, // Invalid
        -65.5,
        20e6,
        18.3,
    );
    assert!(matches!(m, Err(ValidationError::InvalidFrequency(_))));
}
