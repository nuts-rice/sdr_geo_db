// T013: Contract tests for Aggregation
use sdr_db::{aggregate_by_location, Measurement, Coordinate};
use chrono::{Utc, Duration};

#[test]
fn test_aggregate_by_location_groups_correctly() {
    // T013: Aggregate measurements at same location
    let base_time = Utc::now();
    let location = Coordinate::new(37.7749, -122.4194).unwrap();

    let measurements = vec![
        create_measurement_at_location(location, base_time, -60.0),
        create_measurement_at_location(location, base_time + Duration::minutes(5), -65.0),
        create_measurement_at_location(location, base_time + Duration::minutes(10), -70.0),
    ];

    let aggregates = aggregate_by_location(&measurements, 0.0001); // ~10m epsilon

    assert_eq!(aggregates.len(), 1); // All at same location
    assert_eq!(aggregates[0].measurement_count, 3);
    assert_eq!(aggregates[0].power.min, -70.0);
    assert_eq!(aggregates[0].power.max, -60.0);
    assert!((aggregates[0].power.avg - (-65.0)).abs() < 0.1);
}

// Helper function
fn create_measurement_at_location(
    coord: Coordinate,
    timestamp: chrono::DateTime<Utc>,
    power: f64,
) -> Measurement {
    Measurement::new(coord, timestamp, 2.45e9, power, 20e6, 18.0).unwrap()
}
