use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    prelude::Alignment,
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, List, ListItem, Paragraph, Widget},
    Frame,
};

use crate::App;
use crate::Theme;
use sdr_db::spectrum_types::SpectrumFrame;
const BUFFER_SIZE: usize = 4096;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpectrumSource {
    HackRF,
    RTLSDR,
    File,
}

impl SpectrumSource {
    pub fn as_str(&self) -> &str {
        match self {
            SpectrumSource::HackRF => "hackrf",
            SpectrumSource::RTLSDR => "rtlsdr",
            SpectrumSource::File => "file",
        }
    }

    pub fn all() -> Vec<SpectrumSource> {
        vec![
            SpectrumSource::HackRF,
            SpectrumSource::RTLSDR,
            SpectrumSource::File,
        ]
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

    pub fps: usize,
}

impl Default for SpectrumViewerState {
    fn default() -> Self {
        SpectrumViewerState {
            source: SpectrumSource::HackRF,
            center_frequency: 100_000_000.0,
            span: 20_000_000.0,
            frequency_step: 10_000.0,
            spectrum_data: vec![],
            lna_gain: 16,
            vga_gain: 20,
            time: 0.0,
            peak_hold: vec![],
            theme: Theme::default(),
            file_source_select: FileSourceSelectState::default(),
            fps: 15,
        }
    }
}

impl SpectrumViewerState {
    pub fn update_from_frame(&mut self, frame: SpectrumFrame) {
        self.center_frequency = frame.center_freq;
        self.span = frame.span;
        self.spectrum_data = frame
            .data
            .iter()
            .enumerate()
            .map(|(i, &dbm)| {
                let freq = self.center_frequency - (self.span / 2.0)
                    + (i as f64 / frame.data.len() as f64) * self.span;
                (freq, dbm)
            })
            .collect();
        self.time = frame.timestamp as f64;
        self.update_peak_hold();
    }

    fn update_peak_hold(&mut self) {
        for (freq, dbm) in &self.spectrum_data {
            match self.peak_hold.iter_mut().find(|(f, _)| f == freq) {
                Some((_, peak_dbm)) => {
                    if dbm > peak_dbm {
                        *peak_dbm = *dbm;
                    }
                }
                None => {
                    self.peak_hold.push((*freq, *dbm));
                }
            }
        }
    }
}

pub fn render_spectrum_view(
    state: &SpectrumViewerState,
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    connected: bool,
) {
    let x_min = state.center_frequency - (state.span / 2.0);
    let x_max = state.center_frequency + (state.span / 2.0);
    let y_min = -120.0;
    let y_max = 0.0;
    let x_labels = vec![
        format!("{:.1} MHz", x_min / 1_000_000.0),
        format!("{:.1} MHz", (x_min + x_max) / 1_000_000. / 2.),
        format!("{:.1} MHz", x_max / 1_000_000.0),
    ];
    let y_labels = vec![
        format!("{:.0} dBm", y_min),
        format!("{:.0} dBm", (y_min + y_max) / 2.0),
        format!("{:.0} dBm", y_max),
    ];

    let chunks = Layout::vertical([
        Constraint::Length(3), //header
        Constraint::Min(10),   //chart
        Constraint::Length(3), //footer
    ])
    .split(area);
    let status = if connected {
        "● Live"
    } else {
        "○ Connecting..."
    };
    let status_color = if connected {
        theme.primary_color()
    } else {
        theme.accent_color()
    };
    let header_block = Paragraph::new(Line::from(vec![
        Span::styled(
            "Spectrum Viewer",
            Style::default()
                .fg(theme.primary_color())
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" - "),
        Span::styled(status, Style::default().fg(status_color)),
    ]))
    .block(Block::default().borders(Borders::ALL).title("Status"))
    .alignment(Alignment::Center);
    frame.render_widget(header_block, chunks[0]);

    let datasets = vec![
        Dataset::default()
            .name("Spectrum")
            .marker(symbols::Marker::Braille)
            .style(Style::default().fg(theme.secondary_color()))
            .data(&state.spectrum_data)
            .graph_type(GraphType::Line),
        Dataset::default()
            .name("Peak Hold")
            .marker(symbols::Marker::Dot)
            .style(Style::default().fg(theme.primary_color()))
            .data(&state.peak_hold)
            .graph_type(GraphType::Line),
    ];
    let chart = Chart::new(datasets)
        .block(Block::default().borders(Borders::ALL).title("Spectrum"))
        .x_axis(
            Axis::default()
                .title("Frequency (MHz)")
                .style(Style::default().fg(theme.secondary_color()))
                .bounds([x_min, x_max])
                .labels(x_labels)
                .style(Style::default().fg(theme.secondary_color())),
        )
        .y_axis(
            Axis::default()
                .title("Power (dBm)")
                .bounds([y_min, y_max])
                .labels(y_labels)
                .style(Style::default().fg(theme.secondary_color())),
        );

    frame.render_widget(chart, chunks[1]);

    let footer_text = format!(
        "Center: {:.3} MHz | Span: {:.1} MHz | {:?} FPS",
        state.center_frequency / 1_000_000.0,
        state.span / 1_000_000.0,
        state.fps,
    );
    let footer_block = Paragraph::new(footer_text)
        .block(Block::default().borders(Borders::ALL).title("Info"))
        .alignment(Alignment::Center);
    frame.render_widget(footer_block, chunks[2]);
}

fn render_spectrum(state: &SpectrumViewerState, area: Rect, buf: &mut Buffer, theme: &Theme) {
    let x_min = state.center_frequency - (state.span / 2.0);
    let x_max = state.center_frequency + (state.span / 2.0);
    let y_min = -120.0;
    let y_max = 0.0;

    for (i, &(freq, power)) in state.spectrum_data.iter().enumerate() {
        let x_ratio = (freq - x_min) / (x_max - x_min);
        let y_ratio = (power - y_min) / (y_max - y_min);

        let x_pos = area.x + (x_ratio * area.width as f64) as u16;
        let y_pos = area.y + area.height - 1 - (y_ratio * area.height as f64) as u16;

        if x_pos < area.x + area.width && y_pos < area.y + area.height {
            buf.get_mut(x_pos, y_pos)
                .set_symbol("•")
                .set_fg(theme.secondary_color());
        }
    }
}
