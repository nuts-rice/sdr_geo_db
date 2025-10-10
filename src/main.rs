use sdr_db::{create_log, establish_connection, spatial::Coordinate, error::ValidationError};
use clap::Parser;
use dotenvy::dotenv;
use std::env;
use tracing::{info, error};
use tracing_subscriber;

#[derive(Parser, Debug)]
#[command(name = "sdr_db")]
#[command(about = "SDR Database - Collect and store SDR measurements with geospatial data", long_about = None)]
struct Args {
    /// Database URL (or use DATABASE_URL env var)
    #[arg(long, env = "DATABASE_URL")]
    database_url: Option<String>,

    /// Latitude in decimal degrees [-90, 90]
    #[arg(long)]
    latitude: Option<f64>,

    /// Longitude in decimal degrees [-180, 180]
    #[arg(long)]
    longitude: Option<f64>,

    /// Frequency in Hz (must be positive)
    #[arg(long)]
    frequency: Option<i32>,

    /// Bandwidth in Hz
    #[arg(long, default_value_t = 20_000_000)]
    bandwidth: i32,

    /// Callsign or station identifier
    #[arg(long)]
    callsign: Option<String>,

    /// Mode (e.g., FM, AM, SSB)
    #[arg(long, default_value = "UNKNOWN")]
    mode: String,

    /// Power in dBm (optional)
    #[arg(long)]
    power: Option<f64>,

    /// Signal-to-Noise Ratio in dB (optional)
    #[arg(long)]
    snr: Option<f64>,

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
    let database_url = args.database_url
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
            input.trim().parse::<f64>()?
        };

        let longitude = if let Some(lon) = args.longitude {
            lon
        } else {
            println!("Enter longitude [-180, 180]: ");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            if input.trim() == "q" {
                break;
            }
            input.trim().parse::<f64>()?
        };

        // Validate coordinate
        let location = match Coordinate::new(latitude, longitude) {
            Ok(coord) => coord,
            Err(ValidationError::InvalidLatitude(lat)) => {
                error!("Invalid latitude: {}. Must be between -90 and 90", lat);
                continue;
            }
            Err(ValidationError::InvalidLongitude(lon)) => {
                error!("Invalid longitude: {}. Must be between -180 and 180", lon);
                continue;
            }
            Err(e) => {
                error!("Validation error: {}", e);
                continue;
            }
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
            input.trim().parse::<i32>()?
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

        // Use provided values or defaults
        let bandwidth = args.bandwidth;
        let mode = args.mode.clone();
        let power = args.power;
        let snr = args.snr;
        let comment = args.comment.clone();

        // Write to database
        info!("Writing log entry to database...");
        match create_log(
            &mut conn,
            frequency,
            location,
            bandwidth,
            callsign.clone(),
            mode.clone(),
            power,
            snr,
            comment.clone(),
        ) {
            Ok(log) => {
                info!("✓ Log entry created successfully!");
                println!("  ID: {}", log.id);
                println!("  Location: ({}, {})", location.latitude, location.longitude);
                println!("  Frequency: {} Hz", frequency);
                println!("  Callsign: {}", callsign);
                println!("  Mode: {}", mode);
                if let Some(p) = power {
                    println!("  Power: {} dBm", p);
                }
                if let Some(s) = snr {
                    println!("  SNR: {} dB", s);
                }
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
