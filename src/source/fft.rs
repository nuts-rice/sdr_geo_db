use rustfft::{FftPlanner, num_complex::Complex};

pub fn iq_to_spectrum(
    samples: &[Complex<f32>],
    sample_rate: f32,
    center_freq: f32,
) -> Vec<(f64, f64)> {
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(samples.len());
    let mut buffer = samples.to_vec();
    fft.process(&mut buffer);

    buffer
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let freq = center_freq as f64
                + (i as f64 - samples.len() as f64 / 2.0) * sample_rate as f64
                    / samples.len() as f64;
            let power_dbm = 10.0 * (c.norm_sqr() + 1e-10).log10() as f64;
            (freq, power_dbm)
        })
        .collect()
}
