use std::sync::{RwLock, Arc};
use std::time::{SystemTime};

use sled::Db;
use diesel::prelude::*;

mod db;
mod model;
use model::model::{Log, NewLog};
mod schema;
mod spatial;

pub fn create_log(conn: &mut PgConnection, frequency: i32, bandwidth: i32, callsign: String, mode: String, comment: Option<String>,  ) -> Log {
    use crate::schema::log;
    let new_log = NewLog {
        frequency,
        bandwidth,
        callsign: &callsign,
        mode: &mode,
        comment: comment.as_deref(),
        timestamp: SystemTime::now(), 
    };

    diesel::insert_into(log::table)
        .values(&new_log)
        .get_result(conn)
        .expect("Error saving new log")
}



fn main() {
    let database_url = dotenvy::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db = Arc::new(RwLock::new(sled::open("sdr_db").unwrap()));
    println!("Hello, world!");
}
