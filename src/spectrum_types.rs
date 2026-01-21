use serde::Serialize;

#[derive(Serialize, Default, Clone)]
pub struct SpectrumFrame {
    pub center_freq: f64,
    pub span: f64,
    pub data: Vec<f64>,
    pub timestamp: u64,
}

pub fn generate_mock_spectrum(time: f64) -> Vec<f64> {
    let num_points = 1024;
    let mut data = Vec::with_capacity(num_points);
    for i in 0..num_points {
        let freq = i as f64 / num_points as f64 * 100.0;
        let value = (freq - 50.0).powi(2) / 100.0 + (time.sin() * 5.0);
        data.push(value);
    }
    data
}
