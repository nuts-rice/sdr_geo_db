use diesel::FromSqlRow;
use geo::Point as GeoPoint;

pub type Coordinate = (f32, f32);

#[derive(Debug, Clone, FromSqlRow)]
#[diesel(sql_type = Coordinate)]
pub struct DbPoint(pub GeoPoint<f32>);
