use sdr_db::spatial::Coordinate;

#[test]
fn test_coordinate_valid_bounds() {
    // Test cases from contracts/api-contract.md lines 95-99
    assert!(Coordinate::new(0.0, 0.0).is_ok());
    assert!(Coordinate::new(90.0, 180.0).is_ok());
    assert!(Coordinate::new(-90.0, -180.0).is_ok());
}

#[test]
fn test_coordinate_invalid_latitude() {
    assert!(Coordinate::new(91.0, 0.0).is_err());
    assert!(Coordinate::new(-91.0, 0.0).is_err());
}

#[test]
fn test_coordinate_invalid_longitude() {
    assert!(Coordinate::new(0.0, 181.0).is_err());
    assert!(Coordinate::new(0.0, -181.0).is_err());
}
