use diesel::prelude::*;

pub mod error;
pub mod model;
pub mod schema;
pub mod spatial;

pub use error::{DatabaseError, ValidationError};
pub use model::{Log, NewLog};

pub fn create_log(
    conn: &mut PgConnection,
    frequency: i32,
    xcoord: f32,
    ycoord: f32,
    bandwidth: i32,
    callsign: String,
    mode: String,
    power: Option<f64>,
    snr: Option<f64>,
    comment: Option<String>,
) -> Result<Log, diesel::result::Error> {
    use crate::schema::log;
    

    let new_log = NewLog {
        frequency,
        xcoord,
        ycoord,
        callsign: &callsign,
        bandwidth,
        mode: &mode,
        power,
        snr,
        comment: comment.as_deref(),
    };

    diesel::insert_into(log::table)
        .values(&new_log)
        .returning(Log::as_select())
        .get_result(conn)
}

pub fn establish_connection(database_url: &str) -> PgConnection {
    PgConnection::establish(database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}
