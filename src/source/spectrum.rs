use crate::source::SourceError;

/// Trait for sources that provide spectrum data for visualization
///
/// This trait abstracts over different sources of spectrum data (files, live SDR devices)
/// and provides a unified interface for the spectrum viewer UI.
pub trait SpectrumDataSource {
    /// Get spectrum data for a given frequency range
    ///
    /// # Arguments
    /// * `center_freq` - Center frequency in Hz
    /// * `span` - Frequency span in Hz (total width)
    ///
    /// # Returns
    /// Vector of (frequency_hz, power_dbm) tuples
    fn get_spectrum_data(
        &mut self,
        center_freq: f64,
        span: f64,
    ) -> Result<Vec<(f64, f64)>, SourceError>;

    /// Get human-readable information about this source
    fn get_info(&self) -> String;

    /// Set the center frequency for live sources
    /// For file sources, this may have no effect
    fn set_center_frequency(&mut self, freq: f64) -> Result<(), SourceError>;

    /// Get the valid frequency range for this source
    /// Returns (min_freq_hz, max_freq_hz)
    fn get_frequency_range(&self) -> (f64, f64);

    /// Returns true if this is a live streaming source (requires continuous updates)
    fn is_live(&self) -> bool;
}
