use std::{cell::RefCell, io, rc::Rc};

use ratatui::{
    layout::{Alignment, Constraint, Layout},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    Frame, Terminal, DefaultTerminal
};

use ratzilla::{
    event::{KeyCode, KeyEvent},
    DomBackend, WebRenderer,
};

use sdr_db::{LogFormData, SignalMode, };
pub mod components;
use components::geolocate_gridsquare;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FormField {
    Frequency,
    GridSquare,
    Callsign,
    Mode,
    Comment,
    RecordingDuration,
}

impl Default for FormField {
    fn default() -> Self {
        FormField::Frequency
    }
}

impl FormField {
    fn label(&self) -> &str {
        match self {
            FormField::Frequency => "MHz",
            FormField::GridSquare => "Grid square",
            FormField::Callsign => "Callsign",
            FormField::Mode => "Mode",
            FormField::Comment => "Comment",
            FormField::RecordingDuration => "Seconds",
        }
    }

    fn next(&self) -> Self {
        match self {
            FormField::Frequency => FormField::GridSquare,
            FormField::GridSquare => FormField::Callsign,
            FormField::Callsign => FormField::Mode,
            FormField::Mode => FormField::Comment,
            FormField::Comment => FormField::RecordingDuration,
            FormField::RecordingDuration => FormField::Frequency,
        }
    }

    fn previous(&self) -> Self {
        match self {
            FormField::Frequency => FormField::RecordingDuration,
            FormField::GridSquare => FormField::Frequency,
            FormField::Callsign => FormField::GridSquare,
            FormField::Mode => FormField::Callsign,
            FormField::Comment => FormField::Mode,
            FormField::RecordingDuration => FormField::Comment,
        }
    }
}

fn main() -> io::Result<()> {
    let database_url = dotenvy::var("DATABASE_URL").expect("DATABASE_URL must be set ");
    
    let backend = DomBackend::new()?;
    let terminal = Terminal::new(backend)?;
    let state = Rc::new(App::default());

    let event_state = Rc::clone(&state);
    terminal.on_key_event(move |key_event| {
        event_state.handle_events(key_event, &event_state);
    });

    let render_state = Rc::clone(&state);
    terminal.draw_web(move |frame| {
        render_state.render(frame);
    });

    Ok(())
}

#[derive(Default)]
struct App {
    form_data: RefCell<LogFormData>,
    frequency_input: RefCell<String>,
    grid_square_input: RefCell<String>,
    callsign_input: RefCell<String>,
    comment_input: RefCell<String>,
    duration_input: RefCell<String>,
    selected_field: RefCell<FormField>,
    status_message: RefCell<Option<String>>,
    state: RefCell<AppState>,
}


impl App {

    fn run(mut self, terminal: &mut DefaultTerminal, database_url: &str) -> io::Result<()> {
        while self.state == AppState::Running {
            let conn = &mut PgConnection::establish(database_url)?;
        
        loop {
            terminal.draw_web(move |frame| {
                app_rc.render(frame);
            })?;
        }
    }
    }
    fn render(&self, frame: &mut Frame) {
        use ratatui::widgets::*;

        let chunks = Layout::vertical([
            Constraint::Length(3), // Title
            Constraint::Min(0),    // Form
            Constraint::Length(3), // Status
            Constraint::Length(1), // Help
        ])
        .split(frame.area());

        // Title
        let title = Paragraph::new("SDR Database - New Log Entry")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .alignment(Alignment::Center)
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD);
        frame.render_widget(title, chunks[0]);

        // Form
        self.render_form(frame, chunks[1]);

        // Status
        if let Some(ref msg) = *self.status_message.borrow() {
            let status = Paragraph::new(msg.as_str())
                .block(Block::default().borders(Borders::ALL).title("Status"))
                .fg(Color::Yellow)
                .alignment(Alignment::Center);
            frame.render_widget(status, chunks[2]);
        }

        // Help
        let help = Line::from(vec![
            Span::raw("Tab/Shift+Tab: Navigate | "),
            Span::raw("Enter: Submit | "),
            Span::raw("G: Geolocate | "),
            Span::raw("Esc: Clear"),
        ]);
        frame.render_widget(help, chunks[3]);
    }

    fn render_form(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        use ratatui::widgets::*;

        let focused = *self.selected_field.borrow();
        let form = self.form_data.borrow();

        let fields = [
            (FormField::Frequency, self.frequency_input.borrow().clone()),
            (
                FormField::GridSquare,
                self.grid_square_input.borrow().clone(),
            ),
            (FormField::Callsign, self.callsign_input.borrow().clone()),
            (FormField::Mode, format!("{:?}", form.mode)),
            (FormField::Comment, self.comment_input.borrow().clone()),
            (
                FormField::RecordingDuration,
                self.duration_input.borrow().clone(),
            ),
        ];

        let items: Vec<ListItem> = fields
            .iter()
            .map(|(field, value)| {
                let is_focused = *field == focused;
                let style = if is_focused {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                let cursor = if is_focused { ">" } else { " " };
                let line = format!("{} {}: {}", cursor, field.label(), value);
                ListItem::new(line).style(style)
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Form Fields")
                .border_type(BorderType::Rounded),
        );

        frame.render_widget(list, area);
    }

    fn request_geolocation(self: &Rc<Self>) {
        if let Some(geolocation) = geolocate_gridsquare::get_geolocation() {
            let app = Rc::clone(self);
            geolocate_gridsquare::request_gridsquare(&geolocation, move |gridsquare| {
                *app.grid_square_input.borrow_mut() = gridsquare.clone();
                *app.status_message.borrow_mut() =
                    Some(format!("✓ Location detected: {}", gridsquare));
            });
            *self.status_message.borrow_mut() = Some("Requesting location...".to_string());
        } else {
            *self.status_message.borrow_mut() =
                Some("✗ Geolocation not available in this browser".to_string());
        }
    }

    fn handle_events(&self, key_event: KeyEvent, app_rc: &Rc<Self>) {
        match key_event.code {
            KeyCode::Tab => {
                let mut selected = self.selected_field.borrow_mut();
                *selected = if key_event.shift {
                    selected.previous()
                } else {
                    selected.next()
                };
            }
            KeyCode::Enter => {
                self.submit_form();
            }
            KeyCode::Esc => {
                self.clear_form();
            }
            KeyCode::Backspace => {
                self.handle_backspace();
            }
            KeyCode::Char(c) => match c.to_ascii_lowercase() {
                'g' => app_rc.request_geolocation(),
                _ => self.handle_char_input(c),
            },
            KeyCode::Up | KeyCode::Down => {
                if *self.selected_field.borrow() == FormField::Mode {
                    self.cycle_mode(key_event.code == KeyCode::Up);
                }
            }
            _ => {}
        }
    }

    fn handle_char_input(&self, c: char) {
        let focused = *self.selected_field.borrow();
        match focused {
            FormField::Frequency => self.frequency_input.borrow_mut().push(c),
            FormField::GridSquare => self.grid_square_input.borrow_mut().push(c),
            FormField::Callsign => self.callsign_input.borrow_mut().push(c),
            FormField::Comment => self.comment_input.borrow_mut().push(c),
            FormField::RecordingDuration => self.duration_input.borrow_mut().push(c),
            FormField::Mode => {}
        }
    }

    fn handle_backspace(&self) {
        let focused = *self.selected_field.borrow();
        match focused {
            FormField::Frequency => {
                self.frequency_input.borrow_mut().pop();
            }
            FormField::GridSquare => {
                self.grid_square_input.borrow_mut().pop();
            }
            FormField::Callsign => {
                self.callsign_input.borrow_mut().pop();
            }
            FormField::Comment => {
                self.comment_input.borrow_mut().pop();
            }
            FormField::RecordingDuration => {
                self.duration_input.borrow_mut().pop();
            }
            FormField::Mode => {}
        }
    }

    fn cycle_mode(&self, reverse: bool) {
        let mut form = self.form_data.borrow_mut();
        form.mode = match form.mode {
            SignalMode::FM => {
                if reverse {
                    SignalMode::CW
                } else {
                    SignalMode::AM
                }
            }
            SignalMode::AM => {
                if reverse {
                    SignalMode::FM
                } else {
                    SignalMode::USB
                }
            }
            SignalMode::USB => {
                if reverse {
                    SignalMode::AM
                } else {
                    SignalMode::LSB
                }
            }
            SignalMode::LSB => {
                if reverse {
                    SignalMode::USB
                } else {
                    SignalMode::CW
                }
            }
            SignalMode::CW => {
                if reverse {
                    SignalMode::LSB
                } else {
                    SignalMode::FM
                }
            }
        };
    }

    fn submit_form(&self) {
        let freq: Result<f32, _> = self.frequency_input.borrow().parse();
        let dur: Result<f32, _> = self.duration_input.borrow().parse();

        match (freq, dur) {
            (Ok(frequency), Ok(duration)) => {
                let mut form = self.form_data.borrow_mut();
                form.frequency = frequency;
                form.grid_square = self.grid_square_input.borrow().clone();
                form.callsign = self.callsign_input.borrow().clone();
                form.comment = self.comment_input.borrow().clone();
                form.recording_duration = duration;

                match form.validate() {
                    Ok(_) => {
                        *self.status_message.borrow_mut() =
                            Some("✓ Form validated! )".to_string());
                    }
                    Err(e) => {
                        *self.status_message.borrow_mut() = Some(format!("✗ Error: {}", e));
                    }
                }
            }
            _ => {
                *self.status_message.borrow_mut() =
                    Some("✗ Invalid input: check numeric fields".to_string());
            }
        }
    }

    fn clear_form(&self) {
        *self.frequency_input.borrow_mut() = String::new();
        *self.grid_square_input.borrow_mut() = String::new();
        *self.callsign_input.borrow_mut() = String::new();
        *self.comment_input.borrow_mut() = String::new();
        *self.duration_input.borrow_mut() = String::new();
        *self.form_data.borrow_mut() = LogFormData::default();
        *self.status_message.borrow_mut() = Some("Form cleared".to_string());
    }
}
