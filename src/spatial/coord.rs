use crate::error::ValidationError;
use diesel::deserialize::{self, FromSql, FromSqlRow};
use diesel::expression::AsExpression;
use diesel::pg::Pg;
use diesel::serialize::{self, Output, ToSql};
use diesel::sql_types::Record;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::io::Write;

/// WGS84 geographic coordinate
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, AsExpression, FromSqlRow)]
#[diesel(sql_type = Record<(diesel::sql_types::Float8, diesel::sql_types::Float8)>)]
pub struct Coordinate {
    pub latitude: f64,
    pub longitude: f64,
}

// Implement ToSql for inserting Coordinates
impl ToSql<Record<(diesel::sql_types::Float8, diesel::sql_types::Float8)>, Pg> for Coordinate {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        // Write as PostgreSQL composite type: (latitude, longitude)
        write!(out, "({},{})", self.latitude, self.longitude)?;
        Ok(serialize::IsNull::No)
    }
}

// Implement FromSql for reading Coordinates
impl FromSql<Record<(diesel::sql_types::Float8, diesel::sql_types::Float8)>, Pg> for Coordinate {
    fn from_sql(bytes: <Pg as diesel::backend::Backend>::RawValue<'_>) -> deserialize::Result<Self> {
        // Parse PostgreSQL composite type representation
        let bytes_str = <String as FromSql<diesel::sql_types::Text, Pg>>::from_sql(bytes)?;

        // Remove parentheses and split by comma
        let coords_str = bytes_str.trim_matches(|c| c == '(' || c == ')');
        let parts: Vec<&str> = coords_str.split(',').collect();

        if parts.len() != 2 {
            return Err("Invalid coordinate format".into());
        }

        let latitude: f64 = parts[0].trim().parse()
            .map_err(|_| "Invalid latitude value")?;
        let longitude: f64 = parts[1].trim().parse()
            .map_err(|_| "Invalid longitude value")?;

        Coordinate::new(latitude, longitude)
            .map_err(|e| format!("Validation error: {}", e).into())
    }
}

impl Coordinate {
    /// Create a new Coordinate with validation
    ///
    /// # Arguments
    /// * `latitude` - Latitude in decimal degrees [-90, 90]
    /// * `longitude` - Longitude in decimal degrees [-180, 180]
    ///
    /// # Errors
    /// Returns `ValidationError::InvalidLatitude` if latitude is out of bounds
    /// Returns `ValidationError::InvalidLongitude` if longitude is out of bounds
    pub fn new(latitude: f64, longitude: f64) -> Result<Self, ValidationError> {
        // Validate latitude bounds [-90, 90]
        if latitude < -90.0 || latitude > 90.0 {
            return Err(ValidationError::InvalidLatitude(latitude));
        }

        // Validate longitude bounds [-180, 180]
        if longitude < -180.0 || longitude > 180.0 {
            return Err(ValidationError::InvalidLongitude(longitude));
        }

        Ok(Coordinate {
            latitude,
            longitude,
        })
    }
}

impl fmt::Display for Coordinate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.latitude, self.longitude)

    }
}
