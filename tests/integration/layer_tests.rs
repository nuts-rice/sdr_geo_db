use sdr_db::{Layer, Measurement, spatial::Coordinate, BoundingBox};
use chrono::Utc;

fn measurement_at(lat: f64, lon: f64) -> Measurement {
    Measurement::new(
        Coordinate::new(lat, lon).unwrap(),
        Utc::now(),
        2.45e9,
        -65.0,
        20e6,
        18.0,
    ).unwrap()
}

#[test]
fn test_query_by_bbox_filters_correctly() {
    // Test case from contracts/api-contract.md (query_by_bbox section)
    let layer = Layer::new(vec![
        measurement_at(37.0, -122.0),
        measurement_at(38.0, -123.0),
        measurement_at(40.0, -120.0),
    ]);

    let bbox = BoundingBox::new(
        Coordinate::new(36.0, -124.0).unwrap(),
        Coordinate::new(39.0, -121.0).unwrap(),
    ).unwrap();

    let results = layer.query_by_bbox(&bbox);
    assert_eq!(results.len(), 2); // First two measurements
}
