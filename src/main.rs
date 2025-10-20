use clap::Parser;
use dotenvy::dotenv;
use sdr_db::model::model::parse_mode;
use sdr_db::{create_log, establish_connection};
use std::env;
use tracing::{error, info};

#[derive(Parser, Debug)]
#[command(name = "sdr_db")]
#[command(about = "SDR Database - Collect and store SDR measurements with geospatial data", long_about = None)]
struct Args {
    /// Database URL (or use DATABASE_URL env var)
    #[arg(long, env = "DATABASE_URL")]
    database_url: Option<String>,

    /// Latitude in decimal degrees [-90, 90]
    #[arg(long)]
    latitude: Option<f32>,

    /// Longitude in decimal degrees [-180, 180]
    #[arg(long)]
    longitude: Option<f32>,

    /// Frequency in Hz (must be positive)
    #[arg(long)]
    frequency: Option<f32>,

    /// Callsign or station identifier
    #[arg(long)]
    callsign: Option<String>,

    /// Mode (e.g., FM, AM, SSB)
    #[arg(long)]
    mode: String,

    /// Optional comment
    #[arg(long)]
    comment: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing subscriber for logging
    tracing_subscriber::fmt::init();

    // Load environment variables from .env file
    dotenv().ok();

    // Parse command line arguments
    let args = Args::parse();

    // Get database URL from args or environment
    let database_url = args
        .database_url
        .or_else(|| env::var("DATABASE_URL").ok())
        .ok_or("DATABASE_URL must be set")?;

    info!("Connecting to database...");
    let mut conn = establish_connection(&database_url);
    info!("Connected successfully!");

    // Check if running in CLI mode (args provided) or interactive mode
    let cli_mode = args.latitude.is_some() && args.frequency.is_some();

    // Interactive mode: continuously read and write logs
    loop {
        if !cli_mode {
            println!("\n=== SDR Database Entry ===");
            println!("Enter 'q' to quit");
        }

        // Get coordinates
        let latitude = if let Some(lat) = args.latitude {
            lat
        } else {
            println!("Enter latitude [-90, 90]: ");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            if input.trim() == "q" {
                break;
            }
            input.trim().parse::<f32>()?
        };

        tracing::debug!("Latitude entered: {}", latitude);

        let longitude = if let Some(lon) = args.longitude {
            lon
        } else {
            println!("Enter longitude [-180, 180]: ");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            if input.trim() == "q" {
                break;
            }
            input.trim().parse::<f32>()?
        };

        // Get frequency
        let frequency = if let Some(freq) = args.frequency {
            freq
        } else {
            println!("Enter frequency (Hz): ");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            if input.trim() == "q" {
                break;
            }
            input.trim().parse::<f32>()?
        };

        // Get callsign
        let callsign = if let Some(ref cs) = args.callsign {
            cs.clone()
        } else {
            println!("Enter callsign: ");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            if input.trim() == "q" {
                break;
            }
            input.trim().to_string()
        };
        let comment = if let Some(ref c) = args.comment {
            c.clone()
        } else {
            println!("Enter comment (optional): ");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            if input.trim() == "q" {
                break;
            }
            input.trim().to_string()
        };

        let mode = if !args.mode.is_empty() {
            args.mode.clone()
        } else {
            println!("Enter mode (AM, FM, USB, LSB, CW) ");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            if input.trim() == "q" {
                break;
            }
            parse_mode(&input.trim().to_string()).to_string()
        };

        // Write to database
        info!("Writing log entry to database...");
        tracing::debug!(
            "Log details: freq={} Hz, coord = ({}, {}), callsign={}, ",
            frequency,
            latitude,
            longitude,
            callsign
        );
        match create_log(
            &mut conn,
            frequency,
            latitude,
            longitude,
            callsign.clone(),
            mode.clone(),
            Some(comment.clone()),
        ) {
            Ok(log) => {
                info!("✓ Log entry created successfully!");
                println!("  ID: {}", log.id);
                println!("  Location: ({}, {})", latitude, longitude);
                println!("  Frequency: {} Hz", frequency);
                println!("  Callsign: {}", callsign);
                println!("  Mode: {}", mode);
                println!("  Timestamp: {}", log.timestamp);
            }
            Err(e) => {
                error!("Failed to create log entry: {}", e);
                continue;
            }
        }

        // If command line args were provided, exit after one entry
        if cli_mode {
            break;
        }
    }

    info!("Exiting...");
    Ok(())
}
