use dotenvy::dotenv;
use sdr_db::{SignalMode, create_log, establish_connection, spatial::coord::Coordinate};
use std::env;

#[test]
fn test_insert_log() {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let mut conn = establish_connection(&database_url).expect("Failed to connect to database");

    let location = "EM12os".to_string();

    let result = create_log(
        &mut conn,
        146.0, // 146 MHz
        location,
        "ABC".to_string(), // 20 kHz bandwidth
        SignalMode::FM,
        "FM".to_string(),
        50.0,
    );

    match result {
        Ok(log) => {
            println!("Successfully created log with ID: {}", log.id);
            assert!(log.id > 0);
        }
        Err(e) => {
            panic!("Failed to insert log: {:?}", e);
        }
    }
}
