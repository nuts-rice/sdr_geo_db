use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, List, ListItem, Widget},
};

//Monochromatic Style consts
const AMBER_BRIGHT: Color = Color::Rgb(255, 170, 0); // #FFAA00 - live data, highlights
const AMBER_MID: Color = Color::Rgb(170, 102, 0); // #AA6600 - labels, borders
const AMBER_DIM: Color = Color::Rgb(85, 51, 0); // #553300 - grid, backgrounds

const THEMES: [&str; 2] = ["Mojave", "Catppuccin"];

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

/*enum SelectedControl {
    Source,
    LNAGain,
    VGAGain,
}
*/

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

    /// Time parameter for animated signal (in radians)
    pub time: f64,

    /// Peak hold values: Vec of (frequency_hz, power_dbm)
    pub peak_hold: Vec<(f64, f64)>,
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
            time: 0.0,
            peak_hold: Vec::new(),
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

    /// Generate sample spectrum data for testing with time-varying signals
    /// TODO: Replace with actual SDR data from HackRF or file
    fn generate_sample_data(&mut self) {
        let (freq_min, _freq_max) = self.frequency_range();
        let num_points = 200;
        let step = self.span / num_points as f64;
        let gain = (self.lna_gain + self.vga_gain) as f64;

        self.spectrum_data.clear();

        // Calculate moving signal positions based on time
        let signal1_offset = (self.time * 0.5).sin() * self.span * 0.3;
        let signal2_offset = (self.time * 0.3 + 1.5).cos() * self.span * 0.2;

        // Amplitude modulation
        let amplitude_mod1 = 0.7 + 0.3 * (self.time * 0.8).sin();
        let amplitude_mod2 = 0.6 + 0.4 * (self.time * 0.4).cos();

        // Generate noise floor around -70 dBm with some variation
        for i in 0..num_points {
            let freq = freq_min + (i as f64 * step);
            let noise_base = -70.0;

            // Time-varying noise floor
            let noise_variation = ((freq / 1e5).sin() * 5.0)
                + ((freq / 2e5).cos() * 3.0)
                + (self.time * 2.0).sin() * 2.0;
            let noise_adjusted = noise_variation + gain;

            // Primary signal peak that moves with time
            let signal1_center = self.center_frequency + signal1_offset;
            let signal1_peak = if (freq - signal1_center).abs() < 50e3 {
                let distance = (freq - signal1_center).abs();
                20.0 * amplitude_mod1 * (1.0 - distance / 50e3)
            } else {
                0.0
            };

            // Secondary signal peak (narrower, weaker, different movement)
            let signal2_center = self.center_frequency + signal2_offset;
            let signal2_peak = if (freq - signal2_center).abs() < 30e3 {
                let distance = (freq - signal2_center).abs();
                15.0 * amplitude_mod2 * (1.0 - distance / 30e3)
            } else {
                0.0
            };

            // Intermittent third signal (appears and disappears)
            let signal3_visibility = ((self.time * 0.6).sin() + 1.0) / 2.0;
            let signal3_offset = self.span * 0.25;
            let signal3_peak = if (freq - (self.center_frequency - signal3_offset)).abs() < 20e3
                && signal3_visibility > 0.3
            {
                let distance = (freq - (self.center_frequency - signal3_offset)).abs();
                12.0 * signal3_visibility * (1.0 - distance / 20e3)
            } else {
                0.0
            };

            // Apply gain to base noise and signals, then cap at 0 dBm (saturation)
            let power =
                (noise_base + noise_adjusted + signal1_peak + signal2_peak + signal3_peak).min(0.0);

            self.spectrum_data.push((freq, power));
        }
    }

    /// Advance time for animated signals
    pub fn tick(&mut self, delta_time: f64) {
        self.time += delta_time;
        self.generate_sample_data();
        self.update_peak_hold();
    }

    /// Update peak hold values - keep maximum power seen at each frequency
    fn update_peak_hold(&mut self) {
        if self.peak_hold.is_empty() {
            self.peak_hold = self.spectrum_data.clone();
        } else {
            for (i, (freq, power)) in self.spectrum_data.iter().enumerate() {
                if i < self.peak_hold.len() {
                    if *power > self.peak_hold[i].1 {
                        self.peak_hold[i] = (*freq, *power);
                    }
                } else {
                    self.peak_hold.push((*freq, *power));
                }
            }
        }
    }

    /// Reset peak hold values
    pub fn reset_peak_hold(&mut self) {
        self.peak_hold.clear();
    }

    /// Reset time parameter
    pub fn reset_time(&mut self) {
        self.time = 0.0;
        self.generate_sample_data();
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
        Constraint::Length(5), // theme
        Constraint::Min(0),    // Remaining space
    ])
    .split(area);

    // Render source selector
    let sources = SpectrumSource::all();
    let items: Vec<ListItem> = sources
        .iter()
        .map(|source| {
            let style = if *source == state.source {
                Style::default().fg(AMBER_MID).add_modifier(Modifier::BOLD)
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
            .border_style(Style::default().fg(AMBER_MID))
            .style(Style::default().bg(AMBER_DIM)),
    );
    list.render(chunks[0], buf);

    let theme_items: Vec<ListItem> = THEMES
        .iter()
        .map(|theme| {
            let style = Style::default().fg(Color::Gray);
            ListItem::new(*theme).style(style)
        })
        .collect();
    let theme_list = List::new(theme_items).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Theme:")
            .border_style(Style::default().fg(AMBER_MID))
            .style(Style::default().bg(AMBER_DIM)),
    );
    theme_list.render(chunks[2], buf);
    // Render gain settings
    let gain_lines = vec![
        Line::from(format!("LNA gain: {} dB", state.lna_gain)),
        Line::from(format!("VGA gain: {} dB", state.vga_gain)),
    ];

    let gain_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(AMBER_MID))
        .style(Style::default().bg(AMBER_DIM));

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


    // Main spectrum dataset
    let dataset = Dataset::default()
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(AMBER_BRIGHT))
        .data(&data_mhz);

    let (freq_min, freq_max) = state.frequency_range();
    let freq_min_mhz = freq_min / 1e6;
    let freq_max_mhz = freq_max / 1e6;

    // Create x-axis labels (all caps for Fallout aesthetic)
    let x_labels = vec![
        Span::styled(
            format!("{:07.3} MHZ", freq_min_mhz),
            Style::default().fg(AMBER_MID),
        ),
        Span::styled(
            format!("{:07.3} MHZ", (freq_min_mhz + freq_max_mhz) / 2.0),
            Style::default().fg(AMBER_MID),
        ),
        Span::styled(
            format!("{:07.3} MHZ", freq_max_mhz),
            Style::default().fg(AMBER_MID),
        ),
    ];

    // Create y-axis labels
    let y_labels = vec![
        Span::styled("-60", Style::default().fg(AMBER_DIM)),
        Span::styled("-50", Style::default().fg(AMBER_DIM)),
        Span::styled("-40", Style::default().fg(AMBER_DIM)),
        Span::styled("-30", Style::default().fg(AMBER_MID)),
        Span::styled("-20", Style::default().fg(AMBER_MID)),
    ];

    let x_axis = Axis::default()
        .title(Span::styled("FREQUENCY", Style::default().fg(AMBER_MID)))
        .style(Style::default().fg(AMBER_DIM))
        .labels(x_labels)
        .bounds([freq_min_mhz, freq_max_mhz]);

    let y_axis = Axis::default()
        .title(Span::styled("POWER (DBM)", Style::default().fg(AMBER_MID)))
        .style(Style::default().fg(AMBER_DIM))
        .labels(y_labels)
        .bounds([-60.0, -20.0]);

    let chart = Chart::new(vec![dataset])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Span::styled(
                    "[>>SPECTRUM<<]",
                    Style::default().fg(AMBER_BRIGHT),
                ))
                .border_style(Style::default().fg(AMBER_MID))
                .style(Style::default().bg(Color::Black)),
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
