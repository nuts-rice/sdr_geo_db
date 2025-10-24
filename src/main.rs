use sdr_db::model::model::{parse_mode, render};
use sdr_db::{create_log, establish_connection};

use clap::Parser;
use dotenvy::dotenv;
use std::env;
use tracing::{error, info};

use color_eyre::Result;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    widgets::{Block, Paragraph},
};

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

    #[arg(long)]
    recording_duration: Option<f32>,
}

//TODO: Tabs for Creating Logs, View Logs, Spectrum View + Source selector
fn run_tui(mut terminal: DefaultTerminal) -> Result<()> {
    loop {
        terminal.draw(draw)?;
        if let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                break Ok(());
            }
    }
}

fn handle_events() -> std::io::Result<()> {
    todo!()
}

fn draw(frame: &mut Frame) {
    let (title_area, layout) = calculate_layout(frame.area());
    render_title(frame, title_area);
    let block = render_block("Create Log Entry");
}

fn render_title(frame: &mut Frame, area: Rect) {
    frame.render_widget(
        Paragraph::new("SDR DB. Press q to quit")
            .dark_gray()
            .alignment(ratatui::layout::Alignment::Center),
        area,
    );
}

fn render_block(title: &str) -> Block {
    Block::bordered()
        .gray()
        .title(title.bold().into_centered_line())
}

fn calculate_layout(area: Rect) -> (Rect, [Rect; 1]) {
    let main_layout = Layout::vertical([Constraint::Length(3), Constraint::Min(0)]);
    let block_layout = Layout::vertical([Constraint::Max(4); 1]);
    let [title_area, main_area] = main_layout.areas(area);
    let blocks = block_layout.areas(main_area);
    (title_area, blocks)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    color_eyre::install()?;
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
            parse_mode(input.trim()).to_string()
        };
        let recording_duration = if let Some(duration) = args.recording_duration {
            duration
        } else {
            println!("Enter recording duration (in seconds):  ");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            if input.trim() == "q" {
                break;
            }
            input.trim().parse::<f32>()?
        };

        // Write to database
        info!("Writing log entry to database...");
        match create_log(
            &mut conn,
            frequency,
            latitude,
            longitude,
            callsign.clone(),
            mode.clone(),
            Some(comment.clone()),
            recording_duration,
        ) {
            Ok(log) => {
                info!("✓ Log entry created successfully!");
                render(&log);
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
