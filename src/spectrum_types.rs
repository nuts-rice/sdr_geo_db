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
    let center_freq = 100_000_000.; 
    let span = 20_000_000.; 
    let data: Vec<f64> = (0..num_points)
        .map(|i| {
            //Model gaussuian signal + noise
            let x = i as f64 / num_points as f64;
            let noise_floor = -90.0;
            let noise = (time + x * 10.).sin() * 3.;
            let signal1 = 40. * (-((x - 0.3).powi(2)) * 500.).exp();
            let signal2 = 30. * (-((x - 0.7).powi(2)) * 300.).exp();
            let t_mod = (time * 0.1).sin() * 0.2 + 0.5;
            let signal3 = 25. * (-((x - t_mod).powi(2)) * 400.).exp();
            noise_floor + noise + signal1 + signal2 + signal3
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
