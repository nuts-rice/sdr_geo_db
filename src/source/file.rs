use crate::source::{Source, SourceError, spectrum::SpectrumDataSource};
use num_complex::Complex;
use tokio::sync::mpsc;

enum ValidFileExtension {
    WAV,
    MP3,
}

struct FileSource {
    source_path: String,
    file_name: String,
    file_extension: ValidFileExtension,
    file_size_bytes: u64,
}

#[async_trait::async_trait]
impl Source for FileSource {
    async fn next_samples(&mut self) -> Result<Option<Vec<Complex<f32>>>, SourceError> {
        todo!()
    }

    async fn start(&mut self) -> Result<(), SourceError> {
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), SourceError> {
        Ok(())
    }
    fn get_receiver(&mut self) -> &mut mpsc::Receiver<Vec<u8>> {
        todo!()
    }
    fn get_device_info(&self) -> String {
        todo!()
    }

    fn get_center_frequency(&self) -> f32 {
        0.0
    }
}

impl SpectrumDataSource for FileSource {
    fn get_spectrum_data(
        &mut self,
        _center_freq: f64,
        _span: f64,
    ) -> Result<Vec<(f64, f64)>, SourceError> {
        todo!("FileSource is for audio files, use FileSpectrum for spectrum data")
    }

    fn get_info(&self) -> String {
        format!("Audio file: {}", self.file_name)
    }

    fn set_center_frequency(&mut self, _freq: f64) -> Result<(), SourceError> {
        Ok(())
    }

    fn get_frequency_range(&self) -> (f64, f64) {
        (0.0, 0.0)
    }

    fn is_live(&self) -> bool {
        false
    }
}

pub struct FileSpectrum {
    file_path: String,
    /// Cached spectrum data: (frequency_hz, power_dbm)
    data: Vec<(f64, f64)>,
    center_freq: f64,
    span: f64,
}

impl FileSpectrum {
    /// Create a new FileSpectrum from a CSV file
    ///
    /// Expected CSV format:
    /// ```csv
    /// frequency_hz,power_dbm
    /// 162000000.0,-65.2
    /// 162001000.0,-68.1
    /// ```
    pub fn from_csv(file_path: String) -> Result<Self, SourceError> {
        use std::fs::File;
        use std::io::{BufRead, BufReader};

        let file = File::open(&file_path)
            .map_err(|e| SourceError::DeviceError(format!("Failed to open file: {}", e)))?;
        let reader = BufReader::new(file);
        let mut data = Vec::new();

        let mut min_freq = f64::MAX;
        let mut max_freq = f64::MIN;

        // Parse CSV data
        for (line_num, line) in reader.lines().enumerate() {
            let line = line.map_err(|e| {
                SourceError::StreamError(format!("Failed to read line {}: {}", line_num + 1, e))
            })?;

            // Skip comments and empty lines
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }

            // Skip header line
            if line_num == 0 && line.contains("frequency") {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 2 {
                return Err(SourceError::StreamError(format!(
                    "Invalid CSV format at line {}: expected 2 columns",
                    line_num + 1
                )));
            }

            let freq: f64 = parts[0].trim().parse().map_err(|_| {
                SourceError::StreamError(format!("Invalid frequency at line {}", line_num + 1))
            })?;
            let power: f64 = parts[1].trim().parse().map_err(|_| {
                SourceError::StreamError(format!("Invalid power at line {}", line_num + 1))
            })?;

            data.push((freq, power));

            min_freq = min_freq.min(freq);
            max_freq = max_freq.max(freq);
        }

        if data.is_empty() {
            return Err(SourceError::StreamError(
                "No data found in file".to_string(),
            ));
        }

        // Sort by frequency
        data.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        let center_freq = (min_freq + max_freq) / 2.0;
        let span = max_freq - min_freq;

        Ok(Self {
            file_path,
            data,
            center_freq,
            span,
        })
    }

    /// Filter data to a specific frequency range
    fn filter_range(&self, center_freq: f64, span: f64) -> Vec<(f64, f64)> {
        let min_freq = center_freq - span / 2.0;
        let max_freq = center_freq + span / 2.0;

        self.data
            .iter()
            .filter(|(freq, _)| *freq >= min_freq && *freq <= max_freq)
            .copied()
            .collect()
    }
}

impl SpectrumDataSource for FileSpectrum {
    fn get_spectrum_data(
        &mut self,
        center_freq: f64,
        span: f64,
    ) -> Result<Vec<(f64, f64)>, SourceError> {
        Ok(self.filter_range(center_freq, span))
    }

    fn get_info(&self) -> String {
        format!(
            "File: {} ({} points, {:.2} MHz span)",
            self.file_path,
            self.data.len(),
            self.span / 1e6
        )
    }

    fn set_center_frequency(&mut self, _freq: f64) -> Result<(), SourceError> {
        // File sources have fixed frequency range, so this is a no-op
        Ok(())
    }

    fn get_frequency_range(&self) -> (f64, f64) {
        if let (Some(first), Some(last)) = (self.data.first(), self.data.last()) {
            (first.0, last.0)
        } else {
            (0.0, 0.0)
        }
    }

    fn is_live(&self) -> bool {
        false
    }
}
