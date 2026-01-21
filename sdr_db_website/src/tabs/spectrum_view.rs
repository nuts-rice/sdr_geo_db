use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, List, ListItem, Widget},
};


use crate::Theme;

const BUFFER_SIZE: usize = 4096;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpectrumSource {
    HackRF,
    File,
}

impl SpectrumSource {
    pub fn as_str(&self) -> &str {
        match self {
            SpectrumSource::HackRF => "hackrf",
            SpectrumSource::File => "file",
        }
    }

    pub fn all() -> Vec<SpectrumSource> {
        vec![SpectrumSource::HackRF, SpectrumSource::File]
    }
}

#[derive(Debug, Clone, Default)]
pub struct FileSourceSelectState {
    pub visible: bool,
    pub input_path: String,
    pub cursor_pos: usize,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SpectrumViewerState {
    pub source: SpectrumSource,
    pub center_frequency: f64,
    pub span: f64,
    pub frequency_step: f64,
    /// Spectrum data: Vec of (frequency_hz, power_dbm)
    pub spectrum_data: Vec<(f64, f64)>,

    pub lna_gain: usize,
    pub vga_gain: usize,

    pub time: f64,

    /// Peak hold values: Vec (frequency_hz, power_dbm)
    pub peak_hold: Vec<(f64, f64)>,

    // TODO: Replace with WASM-compatible spectrum source
    // pub csv_source: Option<FileSpectrum>,

    pub theme: Theme,

    pub file_source_select: FileSourceSelectState,
}
