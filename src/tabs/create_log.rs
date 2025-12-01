use crate::{Log, model::log::SignalMode};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    symbols,
    text::Line,
    widgets::{Block, Borders, Paragraph, Widget},
};
use serde::Serialize;

const LOG_ENTRY_HEADER_STYLE: ratatui::style::Style = Style::new()
    .fg(Color::Rgb(14, 15, 23))
    .bg(Color::Rgb(54, 68, 96));
const NORMAL_ROW_BG: Color = Color::Rgb(14, 15, 23);

//TODO: should these be Option?
#[derive(Serialize)]
pub struct NewLogInputForm {
    #[serde(skip)]
    focus: LogEntryFocus,
    pub frequency: f32,
    pub grid_square: String,
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
            grid_square: String::new(),
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
            LogEntryFocus::GridSquare => LogEntryFocus::Frequency,
            LogEntryFocus::Callsign => LogEntryFocus::GridSquare,
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
                    let current = self.frequency;
                    let mut current_str = current.to_string();
                    current_str.pop();
                    self.frequency = if current_str.is_empty() {
                        0.0
                    } else {
                        current_str.parse::<f32>().unwrap_or_default()
                    };
                }
            }
            LogEntryFocus::GridSquare => {
                // Handle grid square input (4 or 6 characters: AA00 or AA00aa)
                if let KeyCode::Char(c) = event.code {
                    let len = self.grid_square.len();
                    // Only accept up to 6 characters
                    if len < 6 {
                        let valid_char = match len {
                            0 | 1 => {
                                c.is_ascii_alphabetic()
                                    && c.to_ascii_uppercase().is_ascii_alphabetic()
                            }
                            2 | 3 => c.is_ascii_digit(),
                            4 | 5 => {
                                c.is_ascii_alphabetic()
                                    && c.to_ascii_lowercase().is_ascii_alphabetic()
                            }
                            _ => false,
                        };

                        if valid_char {
                            let formatted_char = match len {
                                0 | 1 => c.to_ascii_uppercase(),
                                4 | 5 => c.to_ascii_lowercase(),
                                _ => c,
                            };
                            self.grid_square.push(formatted_char);
                        }
                    }
                } else if event.code == KeyCode::Backspace {
                    self.grid_square.pop();
                }
            }
            LogEntryFocus::Callsign => {
                if let KeyCode::Char(c) = event.code {
                    self.callsign.push(c);
                } else if event.code == KeyCode::Backspace {
                    self.callsign.pop();
                    self.callsign = if self.callsign.is_empty() {
                        "".to_string()
                    } else {
                        self.callsign.clone()
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
                    self.comment.push(c);
                } else if event.code == KeyCode::Backspace {
                    self.comment.pop();
                    self.comment = if self.comment.is_empty() {
                        "".to_string()
                    } else {
                        self.comment.clone()
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
                    let dur = self.recording_duration;
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

    pub fn get_cursor_position(&self, area: Rect) -> Option<(u16, u16)> {
        let field_y_offset = match self.focus {
            LogEntryFocus::Frequency => 1,
            LogEntryFocus::GridSquare => 2,
            LogEntryFocus::Callsign => 3,
            LogEntryFocus::Mode => 4,
            LogEntryFocus::Comment => 5,
            LogEntryFocus::RecordingDuration => 6,
        };

        let label_len = match self.focus {
            LogEntryFocus::Frequency => "Frequency: ".len(),
            LogEntryFocus::GridSquare => "Grid Square: ".len(),
            LogEntryFocus::Callsign => "Callsign: ".len(),
            LogEntryFocus::Mode => "Mode: ".len(),
            LogEntryFocus::Comment => "Comment: ".len(),
            LogEntryFocus::RecordingDuration => "Recording duration: ".len(),
        };
        let value_len = match self.focus {
            LogEntryFocus::Frequency => self.frequency.to_string().len(),
            LogEntryFocus::GridSquare => self.grid_square.len(),
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
    GridSquare,
    Callsign,
    Mode,
    Comment,
    RecordingDuration,
}

impl LogEntryFocus {
    const fn next(&self) -> Self {
        match self {
            LogEntryFocus::Frequency => LogEntryFocus::GridSquare,
            LogEntryFocus::GridSquare => LogEntryFocus::Callsign,
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
        Constraint::Length(1), // Grid Square
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

    // Render the grid square field with validation color
    let grid_square_display = if form.grid_square.is_empty() {
        "______".to_string()
    } else {
        form.grid_square.clone()
    };

    // Validate grid square length (should be 4 or 6 characters when complete)
    let is_valid_length =
        form.grid_square.len() == 4 || form.grid_square.len() == 6 || form.grid_square.is_empty();
    let grid_square_style = if form.focus == LogEntryFocus::GridSquare {
        Style::default().fg(Color::Yellow).bold()
    } else if !is_valid_length {
        Style::default().fg(Color::Red)
    } else {
        Style::default().fg(Color::White)
    };

    let grid_square_field = format!("Grid Square: {}", grid_square_display);
    Paragraph::new(Line::from(grid_square_field))
        .style(grid_square_style)
        .render(chunks[2], buf);

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
