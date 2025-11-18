/// # Arguments
/// * `lat` - Latitude in decimal degrees (-90 to 90)
/// * `lon` - Longitude in decimal degrees (-180 to 180)
///
/// # Returns
/// A 6-character Maidenhead grid square string (e.g., "FN31pr")
pub fn to_maidenhead(lat: f64, lon: f64) -> Result<String, String> {
    if !(-90.0..=90.0).contains(&lat) {
        return Err("Latitude must be between -90 and 90".to_string());
    }
    if !(-180.0..=180.0).contains(&lon) {
        return Err("Longitude must be between -180 and 180".to_string());
    }

    // Adjust coordinates to 0-based system
    let adj_lon = lon + 180.0;
    let adj_lat = lat + 90.0;

    // Field (first pair) - 20° longitude × 10° latitude
    let field_lon = (adj_lon / 20.0).floor() as u8;
    let field_lat = (adj_lat / 10.0).floor() as u8;

    // Square (second pair) - 2° longitude × 1° latitude
    let square_lon = ((adj_lon % 20.0) / 2.0).floor() as u8;
    let square_lat = ((adj_lat % 10.0) / 1.0).floor() as u8;

    // Subsquare (third pair) - 5' longitude × 2.5' latitude
    let subsquare_lon = ((adj_lon % 2.0) * 12.0).floor() as u8;
    let subsquare_lat = ((adj_lat % 1.0) * 24.0).floor() as u8;

    let grid = format!(
        "{}{}{}{}{}{}",
        (b'A' + field_lon) as char,
        (b'A' + field_lat) as char,
        (b'0' + square_lon) as char,
        (b'0' + square_lat) as char,
        (b'a' + subsquare_lon) as char,
        (b'a' + subsquare_lat) as char,
    );

    Ok(grid)
}

/// Convert Maidenhead grid square to center point coordinates
///
/// # Arguments
/// * `grid` - Maidenhead grid square string (4 or 6 characters)
///
/// # Returns
/// Tuple of (latitude, longitude) for the center of the grid square
pub fn from_maidenhead(grid: &str) -> Result<(f64, f64), String> {
    let grid = grid.to_uppercase();
    let chars: Vec<char> = grid.chars().collect();

    if chars.len() != 4 && chars.len() != 6 {
        return Err("Grid square must be 4 or 6 characters".to_string());
    }

    // Field
    let field_lon = (chars[0] as u8 - b'A') as f64 * 20.0;
    let field_lat = (chars[1] as u8 - b'A') as f64 * 10.0;

    // Square
    let square_lon = (chars[2] as u8 - b'0') as f64 * 2.0;
    let square_lat = (chars[3] as u8 - b'0') as f64 * 1.0;

    let (subsquare_lon, subsquare_lat, center_lon, center_lat) = if chars.len() == 6 {
        // For 6-character grid, calculate subsquare offset and add half-subsquare for center
        let sl = (chars[4].to_ascii_lowercase() as u8 - b'a') as f64 * (2.0 / 24.0);
        let ss = (chars[5].to_ascii_lowercase() as u8 - b'a') as f64 * (1.0 / 24.0);
        // Center is half a subsquare: (2°/24)/2 = 1/24° lon, (1°/24)/2 = 1/48° lat
        (sl, ss, 1.0 / 24.0, 1.0 / 48.0)
    } else {
        // For 4-character grid, no subsquare, add half-square for center
        (0.0, 0.0, 1.0, 0.5)
    };

    // Calculate center point
    let lon = field_lon + square_lon + subsquare_lon - 180.0 + center_lon;
    let lat = field_lat + square_lat + subsquare_lat - 90.0 + center_lat;

    Ok((lat, lon))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_maidenhead() {
        // Test known coordinates
        assert_eq!(to_maidenhead(42.0, -87.0).unwrap(), "EN62ma");
        assert_eq!(to_maidenhead(51.5074, -0.1278).unwrap(), "IO91wm");
    }

    #[test]
    fn test_from_maidenhead() {
        let (lat, lon) = from_maidenhead("FN31pr").unwrap();
        assert!((lat - 41.0).abs() < 1.0);
        assert!((lon - (-73.0)).abs() < 1.0);
    }

    #[test]
    fn test_roundtrip() {
        let original_lat = 40.7128;
        let original_lon = -74.0060;
        let grid = to_maidenhead(original_lat, original_lon).unwrap();
        let (lat, lon) = from_maidenhead(&grid).unwrap();

        // Should be within the same grid square (roughly 0.04° precision)
        assert!((lat - original_lat).abs() < 0.05);
        assert!((lon - original_lon).abs() < 0.1);
    }
}
