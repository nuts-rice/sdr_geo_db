use sdr_db_status::{ClusterMetrics, StatusRenderer, DISPLAY_HEIGHT, DISPLAY_WIDTH};
use std::time::Duration;

#[cfg(feature = "simulator")]
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("SDR_DB Status Monitor");
    println!("=====================");

    #[cfg(feature = "simulator")]
    {
        run_simulator().await?;
    }

    #[cfg(feature = "hardware")]
    {
        run_hardware().await?;
    }

    Ok(())
}

#[cfg(feature = "simulator")]
async fn run_simulator() -> Result<(), Box<dyn std::error::Error>> {
    use embedded_graphics::pixelcolor::Rgb565;
    use embedded_graphics::prelude::*;

    println!("Running in simulator mode...");
    println!("Display size: {}x{}", DISPLAY_WIDTH, DISPLAY_HEIGHT);

    // Create simulator display
    let mut display: SimulatorDisplay<Rgb565> =
        SimulatorDisplay::new(Size::new(DISPLAY_WIDTH, DISPLAY_HEIGHT));

    // Create renderer
    let renderer = StatusRenderer::new();

    // Create simulator window
    let output_settings = OutputSettingsBuilder::new()
        .scale(2)
        .pixel_spacing(1)
        .build();

    let mut window = Window::new("SDR_DB Cluster Status", &output_settings);

    println!("Rendering status display...");

    // For now, use mock metrics
    // TODO: Integrate MetricsCollector when API endpoints are available
    let metrics = create_mock_metrics();

    // Render initial frame
    renderer.render(&mut display, &metrics)?;

    println!("Display rendered successfully!");
    println!("Close the window to exit.");

    // Show in window and handle events
    'running: loop {
        window.update(&display);

        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'running,
                SimulatorEvent::KeyDown { .. } => {
                    // Re-render with updated metrics on keypress
                    let updated_metrics = create_mock_metrics();
                    renderer.render(&mut display, &updated_metrics)?;
                }
                _ => {}
            }
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    println!("Shutting down...");
    Ok(())
}

#[cfg(feature = "hardware")]
async fn run_hardware() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hardware mode not yet implemented!");
    println!("TODO: Initialize TFT display driver (ST7789/ILI9341)");
    println!("TODO: Set up MetricsCollector with API endpoints");
    println!("TODO: Implement main update loop");

    // Example structure for hardware implementation:
    //
    // // Initialize SPI and display driver
    // let spi = rppal::spi::Spi::new(...)?;
    // let display = ST7789::new(spi, dc_pin, rst_pin);
    //
    // // Create metrics collector
    // let api_urls = vec![
    //     "http://192.168.1.101:8080".to_string(),
    //     "http://192.168.1.102:8080".to_string(),
    //     "http://192.168.1.103:8080".to_string(),
    // ];
    // let mut collector = MetricsCollector::new(api_urls);
    //
    // // Create renderer
    // let renderer = StatusRenderer::new();
    //
    // // Main loop
    // loop {
    //     let metrics = collector.collect().await?;
    //     renderer.render(&mut display, &metrics)?;
    //     tokio::time::sleep(Duration::from_secs(10)).await;
    // }

    Ok(())
}

/// Create mock metrics for testing
fn create_mock_metrics() -> ClusterMetrics {
    use chrono::Utc;
    use sdr_db_status::{DatabaseStats, ModeDistribution, NodeStatus, SignalStats, SystemStats};
    use std::time::Duration;

    ClusterMetrics {
        timestamp: Utc::now(),
        nodes: vec![
            NodeStatus {
                name: "node-1".to_string(),
                address: "192.168.1.101".to_string(),
                is_up: true,
                request_rate: 127.0,
                error_rate: 0.02,
                uptime: Duration::from_secs(4 * 24 * 3600 + 12 * 3600), // 4d 12h
            },
            NodeStatus {
                name: "node-2".to_string(),
                address: "192.168.1.102".to_string(),
                is_up: true,
                request_rate: 143.0,
                error_rate: 0.00,
                uptime: Duration::from_secs(4 * 24 * 3600 + 12 * 3600),
            },
            NodeStatus {
                name: "node-3".to_string(),
                address: "192.168.1.103".to_string(),
                is_up: true,
                request_rate: 115.0,
                error_rate: 0.01,
                uptime: Duration::from_secs(4 * 24 * 3600 + 11 * 3600),
            },
        ],
        database: DatabaseStats {
            total_logs: 1_234_567,
            log_rate: 42.0,
            active_connections: 7,
            max_connections: 10,
            avg_latency_ms: 12,
        },
        signals: SignalStats {
            freq_range: (88.5, 2400.0),
            mode_distribution: ModeDistribution {
                fm: 600,
                am: 250,
                usb: 100,
                lsb: 30,
                cw: 20,
            },
            unique_grids: 247,
        },
        system: SystemStats {
            cpu_percent: 35.0,
            memory_used_gb: 2.1,
            memory_total_gb: 4.0,
            cpu_history: vec![30.0, 32.0, 35.0, 38.0, 35.0, 33.0, 35.0],
        },
    }
}
