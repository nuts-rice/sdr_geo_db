use crate::model::model::SignalMode;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Layout, Offset, Rect},
    style::{Color, Style, Stylize},
    symbols,
    text::Line,
    widgets::{Block, Borders, Paragraph, Widget},
};
use serde::Serialize;
use tui_prompts::prelude::*;

/* TODO: theres no prompt for selection yet
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
struct CreateLogState<'a> {
    current_field: LogEntryFocus,
    coordinates_state: TextState<'a>,
    frequency_state: TextState<'a>,
    callsign_state: TextState<'a>,
}
*/
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

impl ModeSelectField {
    const fn new(label: &'static str) -> Self {
        Self { label, value: None }
    }

    fn on_key_press(&mut self, event: KeyEvent) {
        match event.code {
            //Round robin through modes
            KeyCode::Up => {
                self.value = match self.value {
                    Some(SignalMode::AM) => Some(SignalMode::FM),
                    Some(SignalMode::FM) => Some(SignalMode::USB),
                    Some(SignalMode::USB) => Some(SignalMode::LSB),
                    Some(SignalMode::LSB) => Some(SignalMode::CW),
                    Some(SignalMode::CW) => Some(SignalMode::AM),
                    None => Some(SignalMode::AM),
                }
            }
            KeyCode::Down => {
                self.value = match self.value {
                    Some(SignalMode::AM) => Some(SignalMode::CW),
                    Some(SignalMode::CW) => Some(SignalMode::LSB),
                    Some(SignalMode::LSB) => Some(SignalMode::USB),
                    Some(SignalMode::USB) => Some(SignalMode::FM),
                    Some(SignalMode::FM) => Some(SignalMode::AM),
                    None => Some(SignalMode::AM),
                }
            }
            _ => {}
        }
    }
}

impl Widget for ModeSelectField {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::horizontal([
            Constraint::Length(self.label.len() as u16 + 2),
            Constraint::Fill(1),
        ]);
        let chunks = layout.split(area);
        let label = Line::from_iter([self.label, ":"]).bold();
        let value = match self.value {
            Some(v) => Line::from(format!("{:?}", v)),
            None => Line::from("_____"),
        };
        label.render(chunks[0], buf);
        value.render(chunks[1], buf);
    }
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
#[derive(Debug, Serialize)]
struct FrequencyField {
    #[serde(skip)]
    label: &'static str,
    value: Option<f32>,
}
#[derive(Debug, Serialize)]
struct CoordinatesField {
    #[serde(skip)]
    label: &'static str,
    longitude_value: Option<f32>,
    latitude_value: Option<f32>,
}

impl CoordinatesField {
    const fn new(label: &'static str) -> Self {
        Self {
            label,
            longitude_value: Some(0.),
            latitude_value: Some(0.),
        }
    }

    fn cursor_offset(&self, is_latitude: bool) -> Offset {
        let x = if is_latitude {
            self.label.len() + self.latitude_value.unwrap_or(0.).to_string().len() + 4
        } else {
            self.label.len() + self.longitude_value.unwrap_or(0.).to_string().len() + 6
        };
        Offset { x: x as i32, y: 0 }
    }

    fn on_key_press(&mut self, event: KeyEvent, is_latitude: bool) {
        match event.code {
            KeyCode::Char(c) if c.is_ascii_digit() || c == '.' => {
                if is_latitude {
                    let current_value = self.latitude_value.unwrap_or(0.).to_string();
                    let new_value = format!("{}{}", current_value, c);
                    self.latitude_value = new_value.parse::<f32>().ok();
                } else {
                    let current_value = self.longitude_value.unwrap_or(0.).to_string();
                    let new_value = format!("{}{}", current_value, c);
                    self.longitude_value = new_value.parse::<f32>().ok();
                }
            }
            _ => {}
        }
    }
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
            KeyCode::Char(c) if c.is_ascii_digit() || c == '.' => {
                let current_value = self.value.unwrap_or(0.).to_string();
                let new_value = format!("{}{}", current_value, c);
                self.value = new_value.parse::<f32>().ok();
            }
            KeyCode::Backspace => {
                if let Some(current_value) = self.value {
                    let current_str = current_value.to_string();
                    let new_str = current_str
                        .trim_end_matches(|c: char| !c.is_ascii_digit())
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

impl Widget for NumberField {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::horizontal([
            Constraint::Length(self.label.len() as u16 + 2),
            Constraint::Fill(1),
        ]);
        let chunks = layout.split(area);
        let label = Line::from_iter([self.label, ": "]).bold();
        let value = match self.value {
            Some(v) => Line::from(v.to_string()),
            None => Line::from("_____"),
        };
        label.render(chunks[0], buf);
        value.render(chunks[1], buf);
    }
}

//TODO: should these be Option?
#[derive(Serialize)]
pub struct NewLogInputForm {
    #[serde(skip)]
    pub focus: LogEntryFocus,
    pub frequency: f32,
    pub latitude: f32,
    pub longitude: f32,
    pub callsign: String,
    pub mode: SignalMode,
    pub comment: String,
    pub recording_duration: f32,
}

impl Default for NewLogInputForm {
    fn default() -> Self {
        Self::new()
    }
}

impl NewLogInputForm {
    pub fn new() -> Self {
        Self {
            focus: LogEntryFocus::default(),
            frequency: 0.0,
            latitude: 0.0,
            longitude: 0.0,
            callsign: "____".to_string(),
            mode: SignalMode::AM,
            comment: "______".to_string(),
            recording_duration: 0.0,
        }
    }

    pub fn next_field(&mut self) {
        self.focus = self.focus.next();
    }

    pub fn previous_field(&mut self) {
        self.focus = match self.focus {
            LogEntryFocus::Frequency => LogEntryFocus::RecordingDuration,
            LogEntryFocus::Latitude => LogEntryFocus::Frequency,
            LogEntryFocus::Longitude => LogEntryFocus::Latitude,
            LogEntryFocus::Callsign => LogEntryFocus::Longitude,
            LogEntryFocus::Mode => LogEntryFocus::Callsign,
            LogEntryFocus::Comment => LogEntryFocus::Mode,
            LogEntryFocus::RecordingDuration => LogEntryFocus::Comment,
        }
    }

    pub fn handle_key_event(&mut self, event: KeyEvent) {
        match self.focus {
            LogEntryFocus::Frequency => {
                if let KeyCode::Char(c) = event.code {
                    if c.is_ascii_digit() || c == '.' {
                        let current = self.frequency.to_string();
                        let new_val = format!("{}{}", current, c);
                        self.frequency = new_val.parse::<f32>().unwrap_or_default();
                    }
                } else if event.code == KeyCode::Backspace {
                    if let (current) = self.frequency {
                        let mut current_str = current.to_string();
                        current_str.pop();
                        self.frequency = if current_str.is_empty() {
                            0.0
                        } else {
                            current_str.parse::<f32>().unwrap_or_default()
                        };
                    }
                }
            }
            LogEntryFocus::Latitude => {
                if let KeyCode::Char(c) = event.code {
                    if c.is_ascii_digit() || c == '.' || c == '-' {
                        let current = self.latitude.to_string();
                        let new_val = format!("{}{}", current, c);
                        self.latitude = new_val.parse::<f32>().unwrap_or_default();
                    }
                } else if event.code == KeyCode::Backspace {
                    if let (current) = self.latitude {
                        let mut current_str = current.to_string();
                        current_str.pop();
                        self.latitude = if current_str.is_empty() {
                            0.0
                        } else {
                            current_str.parse::<f32>().unwrap_or_default()
                        };
                    }
                }
            }
            LogEntryFocus::Longitude => {
                if let KeyCode::Char(c) = event.code {
                    if c.is_ascii_digit() || c == '.' || c == '-' {
                        let current = self.longitude.to_string();
                        let new_val = format!("{}{}", current, c);
                        self.longitude = new_val.parse::<f32>().unwrap_or_default();
                    }
                } else if event.code == KeyCode::Backspace {
                    if let (current) = self.longitude {
                        let mut current_str = current.to_string();
                        current_str.pop();
                        self.longitude = if current_str.is_empty() {
                            0.0
                        } else {
                            current_str.parse::<f32>().unwrap_or_default()
                        };
                    }
                }
            }
            LogEntryFocus::Callsign => {
                if let KeyCode::Char(c) = event.code {
                    let mut current = self.callsign.clone();
                    current.push(c);
                    self.comment = current;
                } else if event.code == KeyCode::Backspace
                    && let current = self.callsign.clone()
                {
                    let mut new_callsign = current;
                    new_callsign.pop();
                    self.callsign = if new_callsign.is_empty() {
                        "".to_string()
                    } else {
                        new_callsign
                    };
                }
            }
            LogEntryFocus::Mode => match event.code {
                KeyCode::Up | KeyCode::Right => {
                    self.mode = match self.mode {
                        SignalMode::AM => SignalMode::FM,
                        SignalMode::FM => SignalMode::USB,
                        SignalMode::USB => SignalMode::LSB,
                        SignalMode::LSB => SignalMode::CW,
                        SignalMode::CW => SignalMode::AM,
                    };
                }
                KeyCode::Down | KeyCode::Left => {
                    self.mode = match self.mode {
                        SignalMode::AM => SignalMode::CW,
                        SignalMode::CW => SignalMode::LSB,
                        SignalMode::LSB => SignalMode::USB,
                        SignalMode::USB => SignalMode::FM,
                        SignalMode::FM => SignalMode::AM,
                    };
                }
                _ => {}
            },
            LogEntryFocus::Comment => {
                if let KeyCode::Char(c) = event.code {
                    let mut current = self.comment.clone();
                    current.push(c);
                    self.comment = current;
                } else if event.code == KeyCode::Backspace
                    && let current = self.comment.clone()
                {
                    let mut new_comment = current;
                    new_comment.pop();
                    self.comment = if new_comment.is_empty() {
                        "".to_string()
                    } else {
                        new_comment
                    };
                }
            }
            LogEntryFocus::RecordingDuration => {
                if let KeyCode::Char(c) = event.code {
                    if c.is_ascii_digit() || c == '.' {
                        let current = self.recording_duration.to_string();
                        let new_value = format!("{}{}", current, c);
                        self.recording_duration = new_value.parse::<f32>().unwrap_or_default();
                    }
                } else if event.code == KeyCode::Backspace {
                    if let (dur) = self.recording_duration {
                        let mut s = dur.to_string();
                        s.pop();
                        self.recording_duration = if s.is_empty() {
                            0.0
                        } else {
                            s.parse::<f32>().unwrap_or_default()
                        };
                    }
                }
            }
        }
    }

    pub fn get_cursor_position(&self, area: Rect) -> Option<(u16, u16)> {
        let field_y_offset = match self.focus {
            LogEntryFocus::Frequency => 1,
            LogEntryFocus::Latitude => 2,
            LogEntryFocus::Longitude => 3,
            LogEntryFocus::Callsign => 4,
            LogEntryFocus::Mode => 5,
            LogEntryFocus::Comment => 6,
            LogEntryFocus::RecordingDuration => 7,
        };
        let label_len = match self.focus {
            LogEntryFocus::Frequency => "Frequency: ".len(),
            LogEntryFocus::Latitude => "Latitude: ".len(),
            LogEntryFocus::Longitude => "Longitude: ".len(),
            LogEntryFocus::Callsign => "Callsign: ".len(),
            LogEntryFocus::Mode => "Mode: ".len(),
            LogEntryFocus::Comment => "Comment: ".len(),
            LogEntryFocus::RecordingDuration => "Recording duration: ".len(),
        };
        let value_len = match self.focus {
            LogEntryFocus::Frequency => self.frequency.to_string().len(),
            LogEntryFocus::Latitude => self.latitude.to_string().len(),
            LogEntryFocus::Longitude => self.longitude.to_string().len(),
            LogEntryFocus::Callsign => self.callsign.len(),
            LogEntryFocus::Mode => 0, // Mode doesn't show cursor
            LogEntryFocus::Comment => self.comment.len(),
            LogEntryFocus::RecordingDuration => self.recording_duration.to_string().len(),
        };
        Some((
            area.x + label_len as u16 + 2 + value_len as u16,
            area.y + field_y_offset,
        ))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
enum LogEntryFocus {
    #[default]
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

pub fn render_create_log_form(form: &NewLogInputForm, area: Rect, buf: &mut Buffer) {
    let block = Block::new()
        .title(Line::raw("Create Log Entry").bold().centered())
        .borders(Borders::TOP)
        .border_set(symbols::border::EMPTY)
        .border_style(LOG_ENTRY_HEADER_STYLE)
        .bg(NORMAL_ROW_BG);
    let layout = Layout::vertical([
        Constraint::Length(3), // Header
        Constraint::Length(1), // Frequency
        Constraint::Length(1), // Latitude
        Constraint::Length(1), // Longitude
        Constraint::Length(1), // Callsign
        Constraint::Length(1), // Mode
        Constraint::Length(1), // Comment
        Constraint::Length(1), // Recording Duration
        Constraint::Fill(1),   // Rest of space
    ]);
    let chunks = layout.split(area);
    block.render(chunks[0], buf);

    let field_style = |is_focused: bool| {
        if is_focused {
            Style::default().fg(Color::Yellow).bold()
        } else {
            Style::default().fg(Color::White)
        }
    };
    let freq_field = format!("Frequency: {} MHz", form.frequency.to_string());

    let lat_field = format!("Latitude: {} °", form.latitude.to_string());

    Paragraph::new(Line::from(freq_field))
        .style(field_style(form.focus == LogEntryFocus::Frequency))
        .render(chunks[1], buf);

    Paragraph::new(Line::from(lat_field))
        .style(field_style(form.focus == LogEntryFocus::Latitude))
        .render(chunks[2], buf);

    let lon_field = format!("Longitude: {} °", form.longitude.to_string());

    Paragraph::new(Line::from(lon_field))
        .style(field_style(form.focus == LogEntryFocus::Longitude))
        .render(chunks[3], buf);

    let callsign_field = format!("Callsign: {}", form.callsign);

    Paragraph::new(Line::from(callsign_field))
        .style(field_style(form.focus == LogEntryFocus::Callsign))
        .render(chunks[4], buf);

    let mode_field = format!("Mode: {:?}", form.mode);

    Paragraph::new(Line::from(mode_field))
        .style(field_style(form.focus == LogEntryFocus::Mode))
        .render(chunks[5], buf);

    let comment_field = format!("Comment: {}", form.comment);

    Paragraph::new(Line::from(comment_field))
        .style(field_style(form.focus == LogEntryFocus::Comment))
        .render(chunks[6], buf);

    let duration_field = format!(
        "Recording duration: {} seconds",
        form.recording_duration.to_string()
    );

    Paragraph::new(Line::from(duration_field))
        .style(field_style(form.focus == LogEntryFocus::RecordingDuration))
        .render(chunks[7], buf);
}
