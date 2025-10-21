use diesel::prelude::*;

pub mod error;
pub mod model;
pub mod schema;
pub mod source;
pub mod spatial;

pub use error::{DatabaseError, ValidationError};
pub use model::{Log, NewLog};

pub fn create_log(
    conn: &mut PgConnection,
    frequency: f32,
    xcoord: f32,
    ycoord: f32,
    callsign: String,
    mode: String,
    comment: Option<String>,
    recording_duration: f32,
) -> Result<Log, diesel::result::Error> {
    use crate::schema::logs;

    let new_log = NewLog {
        frequency,
        xcoord,
        ycoord,
        callsign: &callsign,
        mode: &mode,
        comment: comment.as_deref(),
        recording_duration,
    };

    diesel::insert_into(logs::table)
        .values(&new_log)
        .returning(Log::as_select())
        .get_result(conn)
}

pub fn establish_connection(database_url: &str) -> PgConnection {
    PgConnection::establish(database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}
