use num_complex::Complex;
use soapysdr::{Device, Direction};
use std::env;

const BUFFER_SIZE: usize = 16384;
const CENTER_FREQ: f64 = 100.0e6;
const SAMPLE_RATE: f64 = 48000.;

fn main() {
    println!("Starting SDR test...\n");

    let filter = env::args().nth(1).unwrap_or_default();
    let devices = soapysdr::enumerate(&filter[..]).expect("Failed to enumerate devices");

    if devices.is_empty() {
        println!("No SDR devices found with filter: {}", filter);
        return;
    }

    println!("Found {} device(s):\n", devices.len());

    let devargs = devices.into_iter().next().unwrap();
    println!("Opening: {}\n", devargs);

    let device = Device::new(devargs).expect("Failed to open device");

    println!("=== Device Info ===");
    if let Ok(info) = device.hardware_info() {
        for (key, value) in &info {
            println!("  {}: {}", key, value);
        }
    }

    println!("\n=== Configuring Device ===");
    println!("  Center frequency: {:.2} MHz", CENTER_FREQ / 1e6);
    println!("  Sample rate: {:.2} MHz", SAMPLE_RATE / 1e6);

    device
        .set_frequency(Direction::Rx, 0, CENTER_FREQ, ())
        .expect("Failed to set frequency");

    device
        .set_sample_rate(Direction::Rx, 0, SAMPLE_RATE)
        .expect("Failed to set sample rate");

    if let Ok(gains) = device.list_gains(Direction::Rx, 0) {
        println!("  Available gains: {:?}", gains);
    }
    let _ = device.set_gain_element(Direction::Rx, 0, "LNA", 24.0);
    let _ = device.set_gain_element(Direction::Rx, 0, "VGA", 20.0);
    let _ = device.set_gain_element(Direction::Rx, 0, "AMP", 0.0);
    println!("  Gains set: LNA=24dB, VGA=20dB, AMP=0dB");

    // Create RX stream
    println!("\n=== Starting RX Stream ===");
    let mut rx_stream = device
        .rx_stream::<Complex<f32>>(&[0])
        .expect("Failed to create RX stream");

    rx_stream.activate(None).expect("Failed to activate stream");

    // Read samples
    let mut buffer = vec![Complex::<f32>::new(0.0, 0.0); BUFFER_SIZE];
    let num_reads = 5;

    println!(
        "  Reading {} buffers of {} samples each...\n",
        num_reads, BUFFER_SIZE
    );

    for i in 0..num_reads {
        let num_samples = rx_stream
            .read(&mut [&mut buffer[..]], 1_000_000)
            .expect("Failed to read samples");

        // Compute statistics
        let magnitudes: Vec<f32> = buffer[..num_samples]
            .iter()
            .map(|c| (c.re * c.re + c.im * c.im).sqrt())
            .collect();

        let max_mag = magnitudes.iter().cloned().fold(0.0f32, f32::max);
        let min_mag = magnitudes.iter().cloned().fold(f32::MAX, f32::min);
        let avg_mag: f32 = magnitudes.iter().sum::<f32>() / magnitudes.len() as f32;

        let peak_db = 20.0 * max_mag.log10();

        println!(
            "  Read #{}: {} samples | mag min/avg/max: {:.4}/{:.4}/{:.4} | peak: {:.1} dBFS",
            i + 1,
            num_samples,
            min_mag,
            avg_mag,
            max_mag,
            peak_db
        );

        if i == 0 {
            println!("  First 5 samples: {:?}", &buffer[..5]);
        }
    }

    rx_stream
        .deactivate(None)
        .expect("Failed to deactivate stream");
    println!("\n=== Stream stopped ===");
}
