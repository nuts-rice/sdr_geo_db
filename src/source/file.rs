use crate::source::{SourceError, spectrum::SpectrumDataSource};
pub struct FileSpectrum {
    file_path: String,
    /// Cached spectrum data: (frequency_hz, power_dbm)
    data: Vec<(f64, f64)>,
    _center_freq: f64,
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

        let _center_freq = (min_freq + max_freq) / 2.0;
        let span = max_freq - min_freq;

        Ok(Self {
            file_path,
            data,
            _center_freq,
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

    fn get_tick_interval_hz(&self) -> f64 {
        1_000_000.0 // 1 MHz tick interval
    }
}
