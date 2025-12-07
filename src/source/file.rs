use crate::source::{SourceError, spectrum::SpectrumDataSource};

#[derive(Debug, Clone)]
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
    /// date, time, hz_low, hz_high, hz_bin_width, num_samples, db, db, db, db, db
    /// ```
    /// Each row represents a single sweep across the frequency range with timestamp.
    /// Power measurements in dB are provided for each frequency bin.
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
            if line.to_lowercase().contains("date") || line.to_lowercase().contains("time") {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() < 7 {
                return Err(SourceError::StreamError(format!(
                    "Invalid CSV format at line {}: expected at least 7 columns (date, time, hz_low, hz_high, hz_bin_width, num_samples, db values...)",
                    line_num + 1
                )));
            }

            // Parse frequency range info (skip date/time at indices 0,1)
            let hz_low: f64 = parts[2].trim().parse().map_err(|_| {
                SourceError::StreamError(format!("Invalid hz_low at line {}", line_num + 1))
            })?;
            let _hz_high: f64 = parts[3].trim().parse().map_err(|_| {
                SourceError::StreamError(format!("Invalid hz_high at line {}", line_num + 1))
            })?;
            let hz_bin_width: f64 = parts[4].trim().parse().map_err(|_| {
                SourceError::StreamError(format!("Invalid hz_bin_width at line {}", line_num + 1))
            })?;
            let _num_samples: usize = parts[5].trim().parse().map_err(|_| {
                SourceError::StreamError(format!("Invalid num_samples at line {}", line_num + 1))
            })?;

            // Parse power values starting from column 6
            for (bin_idx, power_str) in parts[6..].iter().enumerate() {
                let power: f64 = power_str.trim().parse().map_err(|_| {
                    SourceError::StreamError(format!(
                        "Invalid power value at line {}, column {}",
                        line_num + 1,
                        bin_idx + 6
                    ))
                })?;

                // Calculate frequency for this bin
                let freq = hz_low + (bin_idx as f64 * hz_bin_width);

                data.push((freq, power));

                min_freq = min_freq.min(freq);
                max_freq = max_freq.max(freq);
            }
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
