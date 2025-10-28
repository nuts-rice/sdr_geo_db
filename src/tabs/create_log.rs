use crate::model::model::SignalMode;
use ratatui::{
    DefaultTerminal,
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Layout, Offset, Rect},
    style::{Color, Style, Stylize},
    symbols,
    text::Line,
    widgets::{Block, Borders, Paragraph, Tabs, Widget},
};
use serde::Serialize;
const LOG_ENTRY_HEADER_STYLE: ratatui::style::Style = Style::new()
    .fg(Color::Rgb(14, 15, 23))
    .bg(Color::Rgb(54, 68, 96));
const NORMAL_ROW_BG: Color = Color::Rgb(14, 15, 23);
#[derive(Debug, Serialize)]
struct ModeSelectField {
    #[serde(skip)]
    label: &'static str,
    value: Option<SignalMode>,
}

#[derive(Debug, Serialize)]
struct TextField {
    #[serde(skip)]
    label: &'static str,
    value: Option<String>,
}

#[derive(Debug, Serialize)]
struct NumberField {
    #[serde(skip)]
    label: &'static str,
    value: Option<f32>,
}

impl NumberField {
    const fn new(label: &'static str) -> Self {
        Self {
            label,
            value: Some(0.),
        }
    }
    fn on_key_press(&mut self, event: KeyEvent) {
        match event.code {
            KeyCode::Char(c) if c.is_digit(10) || c == '.' => {
                let current_value = self.value.unwrap_or(0.).to_string();
                let new_value = format!("{}{}", current_value, c);
                self.value = new_value.parse::<f32>().ok();
            }
            KeyCode::Backspace => {
                if let Some(current_value) = self.value {
                    let current_str = current_value.to_string();
                    let new_str = current_str
                        .trim_end_matches(|c: char| !c.is_digit(10))
                        .to_string();
                    self.value = new_str.parse::<f32>().ok();
                }
            }
            _ => {}
        }
    }
    fn increment(&mut self) {
        self.value = Some(self.value.unwrap_or(0.) + 1.)
    }

    fn decrement(&mut self) {
        self.value = Some(self.value.unwrap_or(0.) - 1.)
    }

    fn cursor_offset(&self) -> Offset {
        let x = (self.label.len() + self.value.unwrap_or(0.).to_string().len() + 2) as i32;
        Offset { x, y: 0 }
    }
}

//TODO: should these be Option?
#[derive(Serialize)]
struct NewLogInputForm {
    #[serde(skip)]
    focus: LogEntryFocus,
    frequency: Option<f32>,
    latitude: Option<f32>,
    longitude: Option<f32>,
    callsign: Option<String>,
    mode: Option<SignalMode>,
    comment: Option<String>,
    recording_duration: Option<f32>,
}

enum LogEntryFocus {
    Frequency,
    Latitude,
    Longitude,
    Callsign,
    Mode,
    Comment,
    RecordingDuration,
}

impl LogEntryFocus {
    const fn next(&self) -> Self {
        match self {
            LogEntryFocus::Frequency => LogEntryFocus::Latitude,
            LogEntryFocus::Latitude => LogEntryFocus::Longitude,
            LogEntryFocus::Longitude => LogEntryFocus::Callsign,
            LogEntryFocus::Callsign => LogEntryFocus::Mode,
            LogEntryFocus::Mode => LogEntryFocus::Comment,
            LogEntryFocus::Comment => LogEntryFocus::RecordingDuration,
            LogEntryFocus::RecordingDuration => LogEntryFocus::Frequency,
        }
    }
}

fn render_create_log_list(area: Rect, buf: &mut Buffer) {
    let block = Block::new()
        .title(Line::raw("Create Log Entry").bold().centered())
        .borders(Borders::TOP)
        .border_set(symbols::border::EMPTY)
        .border_style(LOG_ENTRY_HEADER_STYLE)
        .bg(NORMAL_ROW_BG);
    let items = vec![
        Line::raw("Frequency: ____ MHz"),
        Line::raw("Latitude: ____ ° "),
        Line::raw("Longitude: ____ °"),
        Line::raw("Callsign: _______ "),
        Line::raw("Mode: ____"),
        Line::raw("Comment: ____________ "),
        Line::raw("Recording duration: _____ seconds"),
    ];
}
