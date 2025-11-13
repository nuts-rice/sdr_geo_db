use crate::{Log, model::model::SignalMode};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Offset, Rect},
    style::{Color, Style, Stylize},
    symbols,
    text::Line,
    widgets::{Block, Borders, Paragraph, Widget},
};
use serde::Serialize;
use tui_prompts::prelude::*;

use tui_input::{Input, backend::crossterm::EventHandler};

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

#[derive(Debug, Clone, Serialize)]
struct CoordinatesField {
    #[serde(skip)]
    label: &'static str,
    #[serde(skip)]
    latitude_input: Input,
    #[serde(skip)]
    longitude_input: Input,
    /// Which input field is currently focused (true = latitude, false = longitude)
    #[serde(skip)]
    latitude_focused: bool,
    /// Whether this field is focused in the parent form
    #[serde(skip)]
    is_focused: bool,
    /// Whether the latitude input contains a valid value
    #[serde(skip)]
    latitude_valid: bool,
    /// Whether the longitude input contains a valid value
    #[serde(skip)]
    longitude_valid: bool,
}

impl CoordinatesField {
    fn new(label: &'static str) -> Self {
        Self {
            label,
            latitude_input: Input::from("0.0"),
            longitude_input: Input::from("0.0"),
            latitude_focused: true,
            is_focused: false,
            latitude_valid: true,
            longitude_valid: true,
        }
    }

    /// Toggle focus between latitude and longitude fields
    fn toggle_focus(&mut self) {
        self.latitude_focused = !self.latitude_focused;
    }

    /// Set focus explicitly
    fn set_focus(&mut self, to_latitude: bool) {
        self.latitude_focused = to_latitude;
    }

    /// Validate coordinate inputs and update validation state
    fn validate(&mut self) {
        self.latitude_valid = self
            .latitude_input
            .value()
            .parse::<f32>()
            .map(|lat| (-90.0..=90.0).contains(&lat))
            .unwrap_or(false);

        self.longitude_valid = self
            .longitude_input
            .value()
            .parse::<f32>()
            .map(|lon| (-180.0..=180.0).contains(&lon))
            .unwrap_or(false);
    }

    /// Handle key events for the currently focused input
    fn on_key_press(&mut self, event: KeyEvent) {
        use crossterm::event::KeyModifiers;

        // Handle comma to switch from latitude to longitude
        if event.code == KeyCode::Char(',') && self.latitude_focused {
            self.set_focus(false);
            return;
        }

        // Handle Ctrl+Left/Right to switch between lat/lon
        if event.modifiers.contains(KeyModifiers::CONTROL) {
            match event.code {
                KeyCode::Right => {
                    self.set_focus(false); // Switch to longitude
                    return;
                }
                KeyCode::Left => {
                    self.set_focus(true); // Switch to latitude
                    return;
                }
                _ => {}
            }
        }

        // Filter input to only allow valid float characters and navigation keys
        let is_valid_key = matches!(event.code,
            KeyCode::Char(c) if c.is_ascii_digit() || c == '.' || c == '-')
            || matches!(
                event.code,
                KeyCode::Backspace
                    | KeyCode::Left
                    | KeyCode::Right
                    | KeyCode::Home
                    | KeyCode::End
                    | KeyCode::Delete
            );

        if !is_valid_key {
            return;
        }

        // Pass event to the appropriate input field
        if self.latitude_focused {
            self.latitude_input
                .handle_event(&crossterm::event::Event::Key(event));
            self.validate();
        } else {
            self.longitude_input
                .handle_event(&crossterm::event::Event::Key(event));
            self.validate();
        }
    }

    /// Get the parsed latitude value
    fn get_latitude(&self) -> Result<f32, std::num::ParseFloatError> {
        self.latitude_input.value().parse::<f32>()
    }

    /// Get the parsed longitude value
    fn get_longitude(&self) -> Result<f32, std::num::ParseFloatError> {
        self.longitude_input.value().parse::<f32>()
    }

    /// Get cursor offset for rendering
    fn cursor_offset(&self) -> Offset {
        let label_len = self.label.len() + 2; // "label: "

        if self.latitude_focused {
            // +1 for opening bracket "["
            let x = label_len + 1 + self.latitude_input.cursor();
            Offset { x: x as i32, y: 0 }
        } else {
            // Account for "[lat_value], " before longitude, then "[" for longitude
            let lat_len = self.latitude_input.value().len();
            let x = label_len + 1 + lat_len + 3 + 1 + self.longitude_input.cursor(); // +1 for "[", +3 for "], ", +1 for "["
            Offset { x: x as i32, y: 0 }
        }
    }
}

impl Widget for CoordinatesField {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::horizontal([
            Constraint::Length(self.label.len() as u16 + 2),
            Constraint::Fill(1),
        ]);
        let chunks = layout.split(area);

        let label = Line::from_iter([self.label, ": "]).bold();

        // Priority: validation error (red) > focused (yellow) > normal (white)
        let lat_style = if !self.latitude_valid {
            Style::default().fg(Color::Red).bold()
        } else if self.is_focused && self.latitude_focused {
            Style::default().fg(Color::Yellow).bold()
        } else {
            Style::default().fg(Color::White)
        };

        let lon_style = if !self.longitude_valid {
            Style::default().fg(Color::Red).bold()
        } else if self.is_focused && !self.latitude_focused {
            Style::default().fg(Color::Yellow).bold()
        } else {
            Style::default().fg(Color::White)
        };

        // Add visual indicators for focused field
        let mut spans = vec![];

        if self.is_focused && self.latitude_focused {
            spans.push(ratatui::text::Span::raw("["));
        }
        spans.push(ratatui::text::Span::styled(
            self.latitude_input.value(),
            lat_style,
        ));
        if self.is_focused && self.latitude_focused {
            spans.push(ratatui::text::Span::raw("]"));
        }

        spans.push(ratatui::text::Span::raw(", "));

        if self.is_focused && !self.latitude_focused {
            spans.push(ratatui::text::Span::raw("["));
        }
        spans.push(ratatui::text::Span::styled(
            self.longitude_input.value(),
            lon_style,
        ));
        if self.is_focused && !self.latitude_focused {
            spans.push(ratatui::text::Span::raw("]"));
        }

        let value = Line::from(spans);

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
    #[serde(skip)]
    pub coordinates_field: CoordinatesField,
    pub callsign: String,
    pub mode: SignalMode,
    pub comment: String,
    pub recording_duration: f32,
    pub created_log: Option<Log>,
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
            coordinates_field: CoordinatesField::new("Coordinates"),
            callsign: "____".to_string(),
            mode: SignalMode::AM,
            comment: "______".to_string(),
            recording_duration: 0.0,
            created_log: None,
        }
    }

    pub fn next_field(&mut self) {
        self.focus = self.focus.next();
    }

    pub fn previous_field(&mut self) {
        self.focus = match self.focus {
            LogEntryFocus::Frequency => LogEntryFocus::RecordingDuration,
            LogEntryFocus::Coordinates => LogEntryFocus::Frequency,
            LogEntryFocus::Callsign => LogEntryFocus::Coordinates,
            LogEntryFocus::Mode => LogEntryFocus::Callsign,
            LogEntryFocus::Comment => LogEntryFocus::Mode,
            LogEntryFocus::RecordingDuration => LogEntryFocus::Comment,
        }
    }

    /// Get the validated latitude value from the coordinates field
    pub fn latitude(&self) -> Result<f32, std::num::ParseFloatError> {
        self.coordinates_field.get_latitude()
    }

    /// Get the validated longitude value from the coordinates field
    pub fn longitude(&self) -> Result<f32, std::num::ParseFloatError> {
        self.coordinates_field.get_longitude()
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
                } else if event.code == KeyCode::Backspace
                    && let current = self.frequency
                {
                    let mut current_str = current.to_string();
                    current_str.pop();
                    self.frequency = if current_str.is_empty() {
                        0.0
                    } else {
                        current_str.parse::<f32>().unwrap_or_default()
                    };
                }
            }
            LogEntryFocus::Coordinates => {
                // Handle coordinates field using tui_input
                self.coordinates_field.on_key_press(event);
            }
            LogEntryFocus::Callsign => {
                if let KeyCode::Char(c) = event.code {
                    let mut current = self.callsign.clone();
                    current.push(c);
                    self.callsign = current;
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
                } else if event.code == KeyCode::Backspace
                    && let dur = self.recording_duration
                {
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
        /*match event.code {
             KeyCode::Enter => {
                 if (self.frequency > 0.0 && self.latitude.abs() <= 90.0 && self.longitude.abs() <= 180.0 && self.recording_duration >= 0.) {
                     match create_log(&mut self.conn, self.frequency, self.latitude, self.longitude, self.callsign, self.mode, Some(self.comment), self.recording_duration)
                     {
                         Ok(log) => {
                 info!("✓ Log entry created successfully!");
                 render(&log);


                         }
             Err(e) => {
                 error!("Failed to create log entry: {}", e);
                 continue;

                 }
             }

             }
        */
    }

    pub fn get_cursor_position(&self, area: Rect) -> Option<(u16, u16)> {
        let field_y_offset = match self.focus {
            LogEntryFocus::Frequency => 1,
            LogEntryFocus::Coordinates => 2,
            LogEntryFocus::Callsign => 3,
            LogEntryFocus::Mode => 4,
            LogEntryFocus::Comment => 5,
            LogEntryFocus::RecordingDuration => 6,
        };

        if matches!(self.focus, LogEntryFocus::Coordinates) {
            let offset = self.coordinates_field.cursor_offset();
            return Some((area.x + offset.x as u16, area.y + field_y_offset));
        }

        let label_len = match self.focus {
            LogEntryFocus::Frequency => "Frequency: ".len(),
            LogEntryFocus::Coordinates => "Coordinates: ".len(),
            LogEntryFocus::Callsign => "Callsign: ".len(),
            LogEntryFocus::Mode => "Mode: ".len(),
            LogEntryFocus::Comment => "Comment: ".len(),
            LogEntryFocus::RecordingDuration => "Recording duration: ".len(),
        };
        let value_len = match self.focus {
            LogEntryFocus::Frequency => self.frequency.to_string().len(),
            LogEntryFocus::Coordinates => 0, // Handled above
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
    Coordinates,
    Callsign,
    Mode,
    Comment,
    RecordingDuration,
}

impl LogEntryFocus {
    const fn next(&self) -> Self {
        match self {
            LogEntryFocus::Frequency => LogEntryFocus::Coordinates,
            LogEntryFocus::Coordinates => LogEntryFocus::Callsign,
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
        Constraint::Length(1), // Coordinates
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
    let freq_field = format!("Frequency: {} MHz", form.frequency);

    Paragraph::new(Line::from(freq_field))
        .style(field_style(form.focus == LogEntryFocus::Frequency))
        .render(chunks[1], buf);

    // Render the CoordinatesField widget
    // We need to clone it because Widget::render takes ownership
    let coord_field = CoordinatesField {
        label: form.coordinates_field.label,
        latitude_input: form.coordinates_field.latitude_input.clone(),
        longitude_input: form.coordinates_field.longitude_input.clone(),
        latitude_focused: form.coordinates_field.latitude_focused,
        is_focused: form.focus == LogEntryFocus::Coordinates,
        latitude_valid: form.coordinates_field.latitude_valid,
        longitude_valid: form.coordinates_field.longitude_valid,
    };

    coord_field.render(chunks[2], buf);

    let callsign_field = format!("Callsign: {}", form.callsign);

    Paragraph::new(Line::from(callsign_field))
        .style(field_style(form.focus == LogEntryFocus::Callsign))
        .render(chunks[3], buf);

    let mode_field = format!("Mode: {:?}", form.mode);

    Paragraph::new(Line::from(mode_field))
        .style(field_style(form.focus == LogEntryFocus::Mode))
        .render(chunks[4], buf);

    let comment_field = format!("Comment: {}", form.comment);

    Paragraph::new(Line::from(comment_field))
        .style(field_style(form.focus == LogEntryFocus::Comment))
        .render(chunks[5], buf);

    let duration_field = format!("Recording duration: {} seconds", form.recording_duration);

    Paragraph::new(Line::from(duration_field))
        .style(field_style(form.focus == LogEntryFocus::RecordingDuration))
        .render(chunks[6], buf);
}
