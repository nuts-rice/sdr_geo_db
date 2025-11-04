use diesel::prelude::*;

pub mod error;
pub mod model;
pub mod schema;
pub mod source;
pub mod spatial;
pub mod tabs;

pub use error::{DatabaseError, ValidationError};
pub use model::{Log, NewLog, model::SignalMode};
pub fn create_log(
    conn: &mut PgConnection,
    frequency: f32,
    xcoord: f32,
    ycoord: f32,
    callsign: String,
    mode: SignalMode,
    comment: Option<String>,
    recording_duration: f32,
) -> Result<Log, diesel::result::Error> {
    use crate::schema::logs;

    let new_log = NewLog {
        frequency,
        xcoord,
        ycoord,
        callsign: &callsign,
        mode: &mode.to_str(),
        comment: comment.as_deref(),
        recording_duration,
    };

    diesel::insert_into(logs::table)
        .values(&new_log)
        .returning(Log::as_select())
        .get_result(conn)
}

pub fn get_logs(conn: &mut PgConnection, limit: i64) -> Result<Vec<Log>, diesel::result::Error> {
    use crate::schema::logs::dsl::*;

    logs.order(timestamp.desc())
        .limit(limit)
        .select(Log::as_select())
        .load(conn)
}

pub fn establish_connection(database_url: &str) -> PgConnection {
    PgConnection::establish(database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}
