// T014-T017: Integration tests from Quickstart scenarios
use sdr_db::{Layer, Measurement, Coordinate, BoundingBox, aggregate_by_location};
use chrono::{Utc, Duration};

#[test]
fn test_scenario_1_load_and_prepare_sdr_data() {
    // T014: Scenario 1 - Create measurements with geolocation

    // Valid measurements should be accepted
    let m1 = Measurement::new(
        Coordinate::new(37.7749, -122.4194).unwrap(),
        Utc::now(),
        2.45e9,
        -65.5,
        20e6,
        18.3,
    );
    assert!(m1.is_ok());

    // Invalid coordinates should be rejected
    let invalid_coord = Coordinate::new(91.0, 0.0);
    assert!(invalid_coord.is_err());
}

#[test]
fn test_scenario_2_create_geospatial_layer() {
    // T015: Scenario 2 - Create layer and verify spatial extent
    let measurements = vec![
        create_test_measurement(37.7749, -122.4194),
        create_test_measurement(37.7849, -122.4094),
    ];

    let layer = Layer::new(measurements);

    // Verify layer has measurements
    assert_eq!(layer.measurement_count(), 2);

    // Verify spatial extent is computed
    let extent = layer.spatial_extent();
    assert!(extent.is_some());
}

#[test]
fn test_scenario_3_query_and_visualize_properties() {
    // T016: Scenario 3 - Spatial bbox query and property filtering
    let measurements = vec![
        create_test_measurement(37.7749, -122.4194),
        create_test_measurement(37.7849, -122.4094),
        create_test_measurement(40.0, -120.0), // Outside bbox
    ];

    let layer = Layer::new(measurements);

    // Spatial query
    let bbox = BoundingBox::new(
        Coordinate::new(37.7, -122.5).unwrap(),
        Coordinate::new(37.8, -122.4).unwrap(),
    ).unwrap();

    let results = layer.query_by_bbox(bbox);
    assert_eq!(results.len(), 2);

    // Power threshold filtering
    let strong_signals = layer.query_by_power_threshold(-70.0);
    assert!(strong_signals.len() > 0);

    // Verify all 5 properties accessible
    for m in layer.measurements() {
        assert!(m.frequency > 0.0);
        assert!(m.power != 0.0);
        assert!(m.bandwidth > 0.0);
        assert!(m.snr != 0.0);
        assert!(m.timestamp <= Utc::now());
    }
}

#[test]
fn test_scenario_4_temporal_aggregation() {
    // T017: Scenario 4 - Temporal aggregation at same location
    let base_time = Utc::now();
    let location = Coordinate::new(37.7749, -122.4194).unwrap();

    let measurements = vec![
        create_measurement_at_time_power(location, base_time, -60.0, 20.0),
        create_measurement_at_time_power(location, base_time + Duration::minutes(5), -65.0, 18.0),
        create_measurement_at_time_power(location, base_time + Duration::minutes(10), -70.0, 15.0),
    ];

    let aggregates = aggregate_by_location(&measurements, 0.0001);

    assert_eq!(aggregates.len(), 1);
    assert_eq!(aggregates[0].measurement_count, 3);

    // Verify min/max/avg for power
    assert_eq!(aggregates[0].power.min, -70.0);
    assert_eq!(aggregates[0].power.max, -60.0);
    assert!((aggregates[0].power.avg - (-65.0)).abs() < 0.1);

    // Verify min/max/avg for SNR
    assert_eq!(aggregates[0].snr.min, 15.0);
    assert_eq!(aggregates[0].snr.max, 20.0);
    assert!((aggregates[0].snr.avg - 17.666).abs() < 0.1);
}

// Helper functions
fn create_test_measurement(lat: f64, lon: f64) -> Measurement {
    let coord = Coordinate::new(lat, lon).unwrap();
    Measurement::new(coord, Utc::now(), 2.45e9, -65.0, 20e6, 18.0).unwrap()
}

fn create_measurement_at_time_power(
    coord: Coordinate,
    timestamp: chrono::DateTime<Utc>,
    power: f64,
    snr: f64,
) -> Measurement {
    Measurement::new(coord, timestamp, 2.45e9, power, 20e6, snr).unwrap()
}
