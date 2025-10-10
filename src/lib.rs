use diesel::prelude::*;
use clap::Parser;

pub mod error;
pub mod model;
pub mod spatial;
pub mod schema;

pub use error::{ValidationError, DatabaseError};    
pub use model::{Log, NewLog};

#[derive(Parser, Debug)]
struct Args {
    #[clap(long)]
    database_url: String,
    #[clap(short, long, default_value_t = 4e6)]
    sample_rate: f64,

}

pub fn create_log(
    conn: &mut PgConnection,
    frequency: i32,
    location: spatial::Coordinate,
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
        location,
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
    PgConnection::establish(database_url).unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}
