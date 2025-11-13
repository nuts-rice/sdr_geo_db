use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, List, ListItem, Paragraph, Widget},
};

/// Source type for spectrum data
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

enum SelectedControl {
    Source,
    LNAGain,
    VGAGain,
}

/// State for the spectrum viewer tab
#[derive(Debug, Clone)]
pub struct SpectrumViewerState {
    /// Selected data source
    pub source: SpectrumSource,
    /// Center frequency in Hz
    pub center_frequency: f64,
    /// Frequency span in Hz (total width of display)
    pub span: f64,
    /// Frequency step for Up/Down navigation in Hz
    pub frequency_step: f64,
    /// Spectrum data: Vec of (frequency_hz, power_dbm)
    pub spectrum_data: Vec<(f64, f64)>,

    pub lna_gain: usize,
    pub vga_gain: usize,
}

impl Default for SpectrumViewerState {
    fn default() -> Self {
        let mut state = Self {
            source: SpectrumSource::HackRF,
            center_frequency: 162.5e6, // 162.5 MHz default
            span: 2.0e6,               // 2 MHz span
            frequency_step: 100e3,     // 100 kHz step
            spectrum_data: Vec::new(),
            lna_gain: 0,
            vga_gain: 0,
        };
        state.generate_sample_data();
        state
    }
}

impl SpectrumViewerState {
    pub fn new(center_frequency: f64, span: f64) -> Self {
        let mut state = Self {
            center_frequency,
            span,
            ..Default::default()
        };
        state.generate_sample_data();
        state
    }

    /// Move frequency up by one step
    pub fn increase_frequency(&mut self) {
        self.center_frequency += self.frequency_step;
        self.generate_sample_data();
    }

    pub fn set_lna_gain(&mut self, gain: usize) {
        self.lna_gain = gain;
        self.generate_sample_data();
    }

    pub fn set_vga_gain(&mut self, gain: usize) {
        self.vga_gain = gain;
        self.generate_sample_data();
    }

    /// Increase LNA gain (0-40 dB in 8 dB steps)
    pub fn increase_lna_gain(&mut self) {
        if self.lna_gain < 40 {
            self.lna_gain = (self.lna_gain + 8).min(40);
            self.generate_sample_data();
        }
    }

    /// Decrease LNA gain
    pub fn decrease_lna_gain(&mut self) {
        if self.lna_gain > 0 {
            self.lna_gain = self.lna_gain.saturating_sub(8);
            self.generate_sample_data();
        }
    }

    /// Increase VGA gain (0-62 dB in 2 dB steps)
    pub fn increase_vga_gain(&mut self) {
        if self.vga_gain < 62 {
            self.vga_gain = (self.vga_gain + 2).min(62);
            self.generate_sample_data();
        }
    }

    /// Decrease VGA gain
    pub fn decrease_vga_gain(&mut self) {
        if self.vga_gain > 0 {
            self.vga_gain = self.vga_gain.saturating_sub(2);
            self.generate_sample_data();
        }
    }

    /// Move frequency down by one step
    pub fn decrease_frequency(&mut self) {
        self.center_frequency -= self.frequency_step;
        self.generate_sample_data();
    }

    /// Toggle between source options
    pub fn toggle_source(&mut self) {
        self.source = match self.source {
            SpectrumSource::HackRF => SpectrumSource::File,
            SpectrumSource::File => SpectrumSource::HackRF,
        };
        // Regenerate data when source changes
        self.generate_sample_data();
    }

    /// Set the frequency span
    pub fn set_span(&mut self, span: f64) {
        self.span = span;
        self.generate_sample_data();
    }

    /// Get the frequency range (min, max) in Hz
    pub fn frequency_range(&self) -> (f64, f64) {
        let half_span = self.span / 2.0;
        (
            self.center_frequency - half_span,
            self.center_frequency + half_span,
        )
    }

    /// Generate sample spectrum data for testing
    /// TODO: Replace with actual SDR data from HackRF or file
    fn generate_sample_data(&mut self) {
        let (freq_min, freq_max) = self.frequency_range();
        let num_points = 200;
        let step = self.span / num_points as f64;
        let gain = (self.lna_gain + self.vga_gain) as f64;

        self.spectrum_data.clear();

        // Generate noise floor around -70 dBm with some variation
        for i in 0..num_points {
            let freq = freq_min + (i as f64 * step);
            let noise_base = -70.0;
            let noise_variation = ((freq / 1e5).sin() * 5.0) + ((freq / 2e5).cos() * 3.0);
            let noise_adjusted = noise_variation + gain;

            // Add a prominent signal peak near center
            let signal_peak = if (freq - self.center_frequency).abs() < 50e3 {
                let distance = (freq - self.center_frequency).abs();

                20.0 * (1.0 - distance / 50e3)
            } else {
                0.0
            };
            // Apply gain to base noise and signal, then cap at 0 dBm (saturation)
            let power = (noise_base + noise_adjusted + signal_peak).min(0.0);

            self.spectrum_data.push((freq, power));
        }
    }

    /// Load spectrum data from external source
    pub fn load_spectrum_data(&mut self, data: Vec<(f64, f64)>) {
        self.spectrum_data = data;
    }
}

/// Render the left panel with source selector and gain settings
fn render_left_panel(state: &SpectrumViewerState, area: Rect, buf: &mut Buffer) {
    // Split left panel vertically: source selector at top, gain settings below
    let chunks = Layout::vertical([
        Constraint::Length(5), // Source selector
        Constraint::Length(4), // Gain settings
        Constraint::Min(0),    // Remaining space
    ])
    .split(area);

    // Render source selector
    let sources = SpectrumSource::all();
    let items: Vec<ListItem> = sources
        .iter()
        .map(|source| {
            let style = if *source == state.source {
                Style::default()
                    .fg(Color::Rgb(138, 173, 244))
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };
            ListItem::new(source.as_str()).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Source:")
            .border_style(Style::default().fg(Color::Rgb(237, 135, 150)))
            .style(Style::default().bg(Color::Rgb(14, 15, 23))),
    );
    list.render(chunks[0], buf);

    // Render gain settings
    let gain_lines = vec![
        Line::from(format!("LNA gain: {} dB", state.lna_gain)),
        Line::from(format!("VGA gain: {} dB", state.vga_gain)),
    ];

    let gain_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(237, 135, 150)))
        .style(Style::default().bg(Color::Rgb(14, 15, 23)));

    let gain_paragraph = ratatui::widgets::Paragraph::new(gain_lines)
        .block(gain_block)
        .style(Style::default().fg(Color::Gray));

    gain_paragraph.render(chunks[1], buf);
}

/// Render the spectrum viewer chart
fn render_spectrum_chart(state: &SpectrumViewerState, area: Rect, buf: &mut Buffer) {
    // Convert frequency to MHz for display
    let data_mhz: Vec<(f64, f64)> = state
        .spectrum_data
        .iter()
        .map(|(freq, power)| (*freq / 1e6, *power))
        .collect();

    let dataset = Dataset::default()
        .name("Spectrum")
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(Color::Cyan))
        .data(&data_mhz);

    let (freq_min, freq_max) = state.frequency_range();
    let freq_min_mhz = freq_min / 1e6;
    let freq_max_mhz = freq_max / 1e6;

    // Create x-axis labels
    let x_labels = vec![
        Span::raw(format!("{:.1} M", freq_min_mhz)),
        Span::raw(format!("{:.1} M", (freq_min_mhz + freq_max_mhz) / 2.0)),
        Span::raw(format!("{:.1} M", freq_max_mhz)),
    ];

    // Create y-axis labels (power in dBm)
    let y_labels = vec![
        Span::raw("-60"),
        Span::raw("-50"),
        Span::raw("-40"),
        Span::raw("-30"),
        Span::raw("-20"),
    ];

    let x_axis = Axis::default()
        .title("Frequency")
        .style(Style::default().fg(Color::Gray))
        .labels(x_labels)
        .bounds([freq_min_mhz, freq_max_mhz]);

    let y_axis = Axis::default()
        .title("Power (dBm)")
        .style(Style::default().fg(Color::Gray))
        .labels(y_labels)
        .bounds([-60.0, -20.0]);

    let chart = Chart::new(vec![dataset])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Chart widget")
                .border_style(Style::default().fg(Color::Rgb(237, 135, 150)))
                .style(Style::default().bg(Color::Rgb(14, 15, 23))),
        )
        .x_axis(x_axis)
        .y_axis(y_axis);

    chart.render(area, buf);
}

/// Render the complete spectrum viewer with selector and chart
pub fn render_spectrum_viewer(state: &SpectrumViewerState, area: Rect, buf: &mut Buffer) {
    // Create horizontal layout: selector panel on left, chart on right
    let chunks = Layout::horizontal([Constraint::Length(20), Constraint::Min(0)]).split(area);

    render_left_panel(state, chunks[0], buf);
    render_spectrum_chart(state, chunks[1], buf);

    // Render footer with instructions
    let footer_area = Rect {
        x: area.x,
        y: area.y + area.height.saturating_sub(1),
        width: area.width,
        height: 1,
    };

    let footer_text =
        Line::raw("Up/Down: Frequency | L/Shift+L: LNA Gain | V/Shift+V: VGA Gain | Tab: Source");
    footer_text.render(footer_area, buf);
}
