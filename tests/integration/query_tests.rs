use sdr_db::{Layer, Measurement, spatial::Coordinate};
use chrono::{Utc, Duration};

fn measurement_with_time(lat: f64, lon: f64, minutes_offset: i64) -> Measurement {
    let base_time = Utc::now();
    Measurement::new(
        Coordinate::new(lat, lon).unwrap(),
        base_time + Duration::minutes(minutes_offset),
        2.45e9,
        -65.0,
        20e6,
        18.0,
    ).unwrap()
}

fn measurement_with_frequency(lat: f64, lon: f64, freq: f64) -> Measurement {
    Measurement::new(
        Coordinate::new(lat, lon).unwrap(),
        Utc::now(),
        freq,
        -65.0,
        20e6,
        18.0,
    ).unwrap()
}

fn measurement_with_power(lat: f64, lon: f64, power: f64) -> Measurement {
    Measurement::new(
        Coordinate::new(lat, lon).unwrap(),
        Utc::now(),
        2.45e9,
        power,
        20e6,
        18.0,
    ).unwrap()
}

#[test]
fn test_query_by_time_range() {
    let base_time = Utc::now();
    let measurements = vec![
        measurement_with_time(37.0, -122.0, -10),
        measurement_with_time(37.0, -122.0, 0),
        measurement_with_time(37.0, -122.0, 10),
    ];

    let layer = Layer::new(measurements);

    let start = base_time + Duration::minutes(-5);
    let end = base_time + Duration::minutes(5);

    let results = layer.query_by_time_range(start, end);
    assert_eq!(results.len(), 1); // Only middle measurement
}

#[test]
fn test_query_by_frequency_range() {
    let measurements = vec![
        measurement_with_frequency(37.0, -122.0, 2.4e9),
        measurement_with_frequency(37.0, -122.0, 2.45e9),
        measurement_with_frequency(37.0, -122.0, 2.5e9),
    ];

    let layer = Layer::new(measurements);

    let results = layer.query_by_frequency_range(2.42e9, 2.48e9);
    assert_eq!(results.len(), 1); // Only middle measurement
}

#[test]
fn test_query_by_power_threshold() {
    let measurements = vec![
        measurement_with_power(37.0, -122.0, -70.0),
        measurement_with_power(37.0, -122.0, -65.0),
        measurement_with_power(37.0, -122.0, -60.0),
    ];

    let layer = Layer::new(measurements);

    let results = layer.query_by_power_threshold(-67.0);
    assert_eq!(results.len(), 2); // Last two measurements
}
