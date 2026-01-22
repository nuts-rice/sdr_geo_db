use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default, Clone)]
pub struct SpectrumFrame {
    pub center_freq: f64,
    pub span: f64,
    pub data: Vec<f64>,
    pub timestamp: u64,
}

pub fn generate_mock_spectrum(time: f64) -> SpectrumFrame {
    let num_points = 1024;
    let center_freq = 162_500_000.;
    let span = 2_000_000.;
    let data: Vec<f64> = (0..num_points)
        .map(|i| {
            let x = i as f64 / num_points as f64;
            let noise = (time + x * 10.).sin() * 5.;
            let signal1 = -30. * ((x * 50. - time).powi(2)).exp();
            let signal2 = -40. * ((x * 80. - time * 0.5).powi(2)).exp();
            let animation = (time * 2. + x * 20.).sin() * 3.;
            noise + signal1 + signal2 + animation
        })
        .collect();
    SpectrumFrame {
        center_freq,
        span,
        data,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64,
    }
}
