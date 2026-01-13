#[cfg(feature = "db")]
use diesel::prelude::*;

pub mod api_types;

pub mod error;
pub mod form;
pub mod maidenhead;
pub mod model;
#[cfg(feature = "db")]
pub mod schema;
#[cfg(any(feature = "sdr", feature = "native-ui"))]
pub mod source;
#[cfg(feature = "db")]
pub mod spatial;
#[cfg(feature = "native-ui")]
pub mod tabs;

pub use error::{DatabaseError, ValidationError};
pub use form::LogFormData;
pub use maidenhead::{from_maidenhead, to_maidenhead};
pub use model::{Log, NewLog, log::SignalMode};
use tracing::error;

#[cfg(feature = "db")]
pub fn create_log(
    conn: &mut PgConnection,
    frequency: f32,
    grid_square: String,
    callsign: String,
    mode: SignalMode,
    comment: String,
    recording_duration: f32,
) -> Result<Log, diesel::result::Error> {
    use crate::schema::logs;
    if comment.len() > 512 {
        return Err(diesel::result::Error::RollbackTransaction);
    }
    if callsign.len() > 32 {
        return Err(diesel::result::Error::RollbackTransaction);
    }
    let grid_len = grid_square.len();
    if grid_len != 4 && grid_len != 6 {
        error!("Invalid grid square: must be 4 or 6 characters");
    }
    if frequency > 0.0 && recording_duration >= 0.0 {
        let new_log = NewLog {
            frequency,
            grid: &grid_square,
            callsign: &callsign,
            mode: mode.to_str(),
            comment: &comment,
            recording_duration,
            timestamp: chrono::Utc::now().naive_utc(),
        };

        diesel::insert_into(logs::table)
            .values(&new_log)
            .returning(Log::as_returning())
            .get_result(conn)
    } else {
        Err(diesel::result::Error::RollbackTransaction)
    }
}

#[cfg(feature = "db")]
pub fn get_logs(conn: &mut PgConnection, limit: i64) -> Result<Vec<Log>, diesel::result::Error> {
    use crate::schema::logs::dsl::*;

    logs.order(timestamp.desc())
        .limit(limit)
        .select(Log::as_select())
        .load(conn)
}

#[cfg(feature = "db")]
pub fn establish_connection(database_url: &str) -> Result<PgConnection, diesel::ConnectionError> {
    PgConnection::establish(database_url).map_err(|e| {
        error!("Error connecting to {}: {}", database_url, e);
        e
    })
}
