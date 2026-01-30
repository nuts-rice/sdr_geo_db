use rustfft::{FftPlanner, num_complex::Complex};

pub fn remove_dc_offset(samples: &mut [Complex<f32>]) {
    if samples.is_empty() {
        return;
    }
    let sum: Complex<f32> = samples.iter().sum();
    let mean = sum / samples.len() as f32;
    for sample in samples.iter_mut() {
        *sample -= mean;
    }
}

pub fn hann_window(size: usize) -> Vec<f32> {
    use std::f32::consts::PI;
    (0..size)
        .map(|n| 0.5 - 0.5 * (2.0 * PI * n as f32 / size as f32).cos())
        .collect()
}

/// Apply a window function to samples
pub fn apply_window(samples: &mut [Complex<f32>], window: &[f32]) {
    for (sample, &w) in samples.iter_mut().zip(window.iter()) {
        *sample *= w;
    }
}

/// Convert FFT output to  dB
pub fn fft_to_power_db(fft_output: &[Complex<f32>]) -> Vec<f64> {
    fft_output
        .iter()
        .map(|c| {
            let power = c.norm_sqr();
            10.0 * (power as f64 + 1e-12).log10()
        })
        .collect()
}

pub fn fft_shift<T: Copy>(data: &mut [T]) {
    let n = data.len();
    if n < 2 {
        return;
    }
    let mid = n / 2;
    data.rotate_left(mid);
}

pub fn frequency_axis(num_bins: usize, sample_rate: f64, center_freq: f64) -> Vec<f64> {
    let bin_width = sample_rate / num_bins as f64;
    (0..num_bins)
        .map(|i| {
            let offset = (i as f64 - num_bins as f64 / 2.0) * bin_width;
            center_freq + offset
        })
        .collect()
}

/// Full pipeline: IQ samples -> spectrum (frequency, power_db) pairs
pub fn iq_to_spectrum(
    samples: &[Complex<f32>],
    planner: &mut FftPlanner<f32>,
    sample_rate: f32,
    center_freq: f32,
) -> Vec<(f64, f64)> {
    let mut buffer = samples.to_vec();

    remove_dc_offset(&mut buffer);
    let window = hann_window(buffer.len());
    apply_window(&mut buffer, &window);

    // fft
    let fft = planner.plan_fft_forward(buffer.len());
    fft.process(&mut buffer);

    let mut power_db = fft_to_power_db(&buffer);
    fft_shift(&mut power_db);

    let freqs = frequency_axis(buffer.len(), sample_rate as f64, center_freq as f64);

    freqs.into_iter().zip(power_db).collect()
}

pub fn iq_to_power_db(samples: &[Complex<f32>], planner: &mut FftPlanner<f32>) -> Vec<f64> {
    let mut buffer = samples.to_vec();

    remove_dc_offset(&mut buffer);
    let window = hann_window(buffer.len());
    apply_window(&mut buffer, &window);

    let fft = planner.plan_fft_forward(buffer.len());
    fft.process(&mut buffer);

    let mut power_db = fft_to_power_db(&buffer);
    fft_shift(&mut power_db);
    power_db
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dc_offset_removal() {
        let mut samples = vec![
            Complex::new(10.0, 0.0),
            Complex::new(12.0, 0.0),
            Complex::new(8.0, 0.0),
        ];
        remove_dc_offset(&mut samples);
        let sum: Complex<f32> = samples.iter().sum();
        assert!(sum.re.abs() < 1e-6);
    }

    #[test]
    fn test_hann_window_endpoints() {
        let window = hann_window(4);
        // Hann window should be ~0 at endpoints
        assert!(window[0] < 0.01);
    }

    #[test]
    fn test_fft_shift() {
        let mut data = vec![0, 1, 2, 3, 4, 5, 6, 7];
        fft_shift(&mut data);
        assert_eq!(data, vec![4, 5, 6, 7, 0, 1, 2, 3]);
    }

    #[test]
    fn test_frequency_axis() {
        let freqs = frequency_axis(4, 1000.0, 100.0);
        // Should be centered at 100 Hz with 250 Hz bins
        assert_eq!(freqs.len(), 4);
        assert!((freqs[2] - 100.0).abs() < 1e-6); // DC at center
    }
}
