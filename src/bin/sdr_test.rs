use num_complex::Complex;
use rustfft::FftPlanner;
use soapysdr::{Device, Direction};
use std::env;

const BUFFER_SIZE: usize = 16384;
const CENTER_FREQ: f64 = 162.548e6;
const SAMPLE_RATE: f64 = 2.0e6;

// ./target/debug/sdr_test "driver=hackrf"
fn main() {
    println!("Starting SDR test...\n");

    let filter = env::args().nth(1).unwrap_or_else(|| "".to_string());
    let center_freq = env::args()
        .nth(2)
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(CENTER_FREQ);
    let devices = soapysdr::enumerate(&filter[..]).expect("Failed to enumerate devices");
    let mut planner = FftPlanner::<f32>::new();
    let _fft = planner.plan_fft_forward(BUFFER_SIZE);
    let window: Vec<f32> = (0..BUFFER_SIZE)
        .map(|n| 0.5 - 0.5 * (2.0 * std::f32::consts::PI * n as f32 / BUFFER_SIZE as f32).cos())
        .collect();

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
    println!("  Center frequency: {:.2} MHz", center_freq / 1e6);
    println!("  Sample rate: {:.2} MHz", SAMPLE_RATE / 1e6);

    device
        .set_frequency(Direction::Rx, 0, center_freq, ())
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
        // Remove DC offset (subtract mean)
        let mean: Complex<f32> =
            buffer[..num_samples].iter().sum::<Complex<f32>>() / num_samples as f32;
        for n in 0..num_samples {
            buffer[n] = buffer[n] - mean;
        }

        // Apply window
        for n in 0..num_samples {
            buffer[n] = buffer[n] * window[n];
        }

        _fft.process(&mut buffer);
        let spectrum: Vec<f32> = buffer
            .iter()
            .map(|c| 20.0 * (c.re * c.re + c.im * c.im).sqrt().log10())
            .collect();
        let n = spectrum.len();
        let peak_bin = spectrum
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(i, _)| i)
            .unwrap();
        let bin_shifted = if peak_bin < n / 2 {
            peak_bin + n / 2
        } else {
            peak_bin - n / 2
        };
        let freq_offset = (bin_shifted as f64 - n as f64 / 2.0) * SAMPLE_RATE / n as f64;
        let peak_freq = center_freq + freq_offset;
        let peak_db = spectrum[peak_bin];
        println!(
            "  Peak: {:.1} dB @ {:.4} MHz (offset: {:.0} Hz, bin {})",
            peak_db,
            peak_freq / 1e6,
            freq_offset,
            peak_bin
        );

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
