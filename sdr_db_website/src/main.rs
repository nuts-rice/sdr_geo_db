use std::{cell::RefCell, io, rc::Rc};

use ratatui::{
    layout::{Alignment, Constraint, Layout},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    Frame, Terminal,
};

use ratzilla::{
    event::{KeyCode, KeyEvent},
    DomBackend, WebRenderer,
};

use gloo_timers::future::TimeoutFuture;
use sdr_db::{LogFormData, SignalMode};
use wasm_bindgen_futures::spawn_local;
use web_sys::window;

pub mod api_client;
pub mod components;
pub mod tabs;

use tabs::view_logs::{create_table, create_header, ViewLogsState};    

use components::geolocate_gridsquare;


enum ActiveTab {
    NewLog,
    SpectrumView,  //Show as under construction
    ViewLogs,
}

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
    let backend = DomBackend::new()?;
    let terminal = Terminal::new(backend)?;
    let state = Rc::new(App::new(App::get_api_base_url()));

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

struct App {
    base_url: String,
    form_data: RefCell<LogFormData>,
    frequency_input: RefCell<String>,
    grid_square_input: RefCell<String>,
    callsign_input: RefCell<String>,
    comment_input: RefCell<String>,
    duration_input: RefCell<String>,
    selected_field: RefCell<FormField>,
    status_message: RefCell<Option<String>>,
    active_tab: RefCell<ActiveTab>,
    view_log_state: RefCell<ViewLogsState>,
}

impl App {
    fn new(base_url: String) -> Self {
        Self {
            base_url,
            form_data: RefCell::new(LogFormData::default()),
            frequency_input: RefCell::new(String::new()),
            grid_square_input: RefCell::new(String::new()),
            callsign_input: RefCell::new(String::new()),
            comment_input: RefCell::new(String::new()),
            duration_input: RefCell::new(String::new()),
            selected_field: RefCell::new(FormField::default()),
            status_message: RefCell::new(None),
            active_tab: RefCell::new(ActiveTab::NewLog),
            view_log_state: RefCell::new(ViewLogsState::new()),
        }
    }

    fn render(&self, frame: &mut Frame) {
        use ratatui::widgets::*;

        let chunks = Layout::vertical([
            Constraint::Length(3), // Tabs
            Constraint::Length(3), // Title
            Constraint::Min(0),    // Content
            Constraint::Length(3), // Status
            Constraint::Length(1), // Help
        ])
        .split(frame.area());

        // Tab bar
        let tabs = vec!["New Log", "Spectrum View", "View Logs"];
        let tab_titles: Vec<Span> = tabs
            .iter()
            .map(|t| Span::styled(*t, Style::default().fg(Color::Cyan)))
            .collect();
        let active_tab_idx = match *self.active_tab.borrow() {
            ActiveTab::NewLog => 0,
            ActiveTab::SpectrumView => 1,
            ActiveTab::ViewLogs => 2,
        };
        let tabs_widget = Tabs::new(tab_titles)
            .select(active_tab_idx)
            .block(Block::default().borders(Borders::ALL).title("Tabs"))
            .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .divider(Span::raw("|"));
        frame.render_widget(tabs_widget, chunks[0]);

        // Title (changes per tab)
        let title_text = match *self.active_tab.borrow() {
            ActiveTab::NewLog => "SDR Database - New Log Entry",
            ActiveTab::SpectrumView => "SDR Database - Spectrum View",
            ActiveTab::ViewLogs => "SDR Database - View Logs",
        };
        let title = Paragraph::new(title_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .alignment(Alignment::Center)
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD);
        frame.render_widget(title, chunks[1]);

        // Content (switches based on active tab)
        match *self.active_tab.borrow() {
            ActiveTab::NewLog => {
                self.render_form(frame, chunks[2]);
            }
            ActiveTab::SpectrumView => {
                self.render_under_construction(frame, chunks[2]);
            }
            ActiveTab::ViewLogs => {
                self.render_logs_table(frame, chunks[2]);
            }
        }

        // Status (contextual per tab)
        let (status_title, status_msg, status_color) = match *self.active_tab.borrow() {
            ActiveTab::NewLog => {
                let msg = self.status_message.borrow().clone().unwrap_or_default();
                ("Status", msg, Color::Yellow)
            }
            ActiveTab::ViewLogs => {
                let state = self.view_log_state.borrow();
                let count = state.logs.len();
                let selected = state.selected_index + 1;
                let msg = if count == 0 {
                    "No logs loaded • Press R to refresh".to_string()
                } else {
                    format!("📋 {} logs loaded • Viewing {}/{}", count, selected, count)
                };
                ("Logs", msg, Color::Cyan)
            }
            ActiveTab::SpectrumView => {
                ("Info", "🚧 Coming soon...".to_string(), Color::Magenta)
            }
        };
        let status = Paragraph::new(status_msg)
            .block(Block::default().borders(Borders::ALL).title(status_title))
            .fg(status_color)
            .alignment(Alignment::Center);
        frame.render_widget(status, chunks[3]);

        // Help (changes per tab)
        let help_text = match *self.active_tab.borrow() {
            ActiveTab::NewLog => "Tab: Navigate | Up/Down: Mode | Enter: Submit | G: Geolocate | ←/→: Tab | Esc: Clear",
            ActiveTab::SpectrumView => "←/→: Change Tab | 🚧 Under Construction",
            ActiveTab::ViewLogs => "↑/↓: Scroll | R: Refresh | ←/→: Tab | Enter: Details",
        };
        let help = Line::from(vec![
            Span::raw(help_text),
            Span::raw("  |                                                               Made with 🦀💜🦀 in Colorado"),
        ]);
        frame.render_widget(help, chunks[4]);
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

    fn render_under_construction(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        use ratatui::widgets::*;

        let text = vec![
            Line::from(""),
            Line::from(Span::styled("🚧 Under Construction 🚧", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from("Spectrum View coming soon!"),
        ];

        let paragraph = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title("Spectrum View"))
            .alignment(Alignment::Center);

        frame.render_widget(paragraph, area);
    }

    fn render_logs_table(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        use ratatui::widgets::*;
        use tabs::view_logs::TableTheme;

        let state = self.view_log_state.borrow();
        let theme = TableTheme::default();

        if state.logs.is_empty() {
            let text = vec![
                Line::from(""),
                Line::from("No logs loaded."),
                Line::from(""),
                Line::from(Span::styled("Press 'R' to refresh", Style::default().fg(Color::Yellow))),
            ];
            let paragraph = Paragraph::new(text)
                .block(Block::default().borders(Borders::ALL).title("View Logs"))
                .alignment(Alignment::Center);
            frame.render_widget(paragraph, area);
            return;
        }

        let header = create_header(&theme);
        let rows = tabs::view_logs::create_rows(&state, &theme);
        let table = create_table(header, rows, &theme);

        frame.render_widget(table, area);
    }

    fn set_status_with_auto_clear(self: &Rc<Self>, message: String, delay_ms: u32) {
        *self.status_message.borrow_mut() = Some(message);

        let app = Rc::clone(self);
        spawn_local(async move {
            TimeoutFuture::new(delay_ms).await;
            *app.status_message.borrow_mut() = None;
        });
    }

    fn request_geolocation(self: &Rc<Self>) {
        if let Some(geolocation) = geolocate_gridsquare::get_geolocation() {
            self.set_status_with_auto_clear("Requesting location...".to_string(), 2000);
            let app = Rc::clone(self);
            geolocate_gridsquare::request_gridsquare(&geolocation, move |gridsquare| {
                *app.grid_square_input.borrow_mut() = gridsquare.clone();
                app.set_status_with_auto_clear(
                    format!("✓ Grid square set to {}", gridsquare),
                    3000,
                );
            });

            *self.status_message.borrow_mut() = Some("Requesting location...".to_string());

            //std::thread::sleep(std::time::Duration::from_secs(2));
        } else {
            self.set_status_with_auto_clear("✗ Geolocation not available".to_string(), 3000);
        }
    }

    fn fetch_logs(self: &Rc<Self>) {
        let app = Rc::clone(self);
        let url = format!("{}/logs", self.base_url);

        spawn_local(async move {
            match api_client::get_logs_async(&url).await {
                Ok(logs) => {
                    app.view_log_state.borrow_mut().refresh_logs(logs);
                }
                Err(e) => {
                    // Could show error in status, but ViewLogs has its own status display
                    web_sys::console::log_1(&format!("Failed to fetch logs: {:?}", e).into());
                }
            }
        });
    }

    fn get_api_base_url() -> String {
        let window = window().expect("should have a window");
        let location = window.location();
        let hostname = location.hostname().unwrap_or_default();

        if hostname.contains("localhost") || hostname.starts_with("127.0.0.1") {
            "http://localhost:3000".to_string()
        } else {
            "https://sdr-db-api.fly.dev".to_string()
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
            KeyCode::Left | KeyCode::Right => {
                self.cycle_tabs(key_event.code == KeyCode::Left);
            }
            KeyCode::Enter => {
                *self.status_message.borrow_mut() = Some("Submitting...".to_string());

                let app_rc = Rc::clone(app_rc);

                spawn_local(async move {
                    app_rc.submit_form(&app_rc.base_url).await;
                });
            }
            KeyCode::Esc => {
                self.clear_form();
            }
            KeyCode::Backspace => {
                self.handle_backspace();
            }
            KeyCode::Char(c) => match c.to_ascii_lowercase() {
                'g' => app_rc.request_geolocation(),
                'r' => {
                    if matches!(*self.active_tab.borrow(), ActiveTab::ViewLogs) {
                        app_rc.fetch_logs();
                    }
                }
                _ => self.handle_char_input(c),
            },
            KeyCode::Up | KeyCode::Down => {
                // Handle scroll in ViewLogs tab
                if matches!(*self.active_tab.borrow(), ActiveTab::ViewLogs) {
                    let mut state = self.view_log_state.borrow_mut();
                    if key_event.code == KeyCode::Up {
                        state.select_previous();
                    } else {
                        state.select_next();
                    }
                } else if *self.selected_field.borrow() == FormField::Mode {
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

    fn handle_active_tab_change(&self, new_tab: ActiveTab) {
        *self.active_tab.borrow_mut() = new_tab;
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

    fn cycle_tabs(&self, reverse: bool) {
        let mut active_tab = self.active_tab.borrow_mut();
        *active_tab = match *active_tab {
            ActiveTab::NewLog => {
                if reverse {
                    ActiveTab::ViewLogs
                } else {
                    ActiveTab::SpectrumView
                }
            }
            ActiveTab::SpectrumView => {
                if reverse {
                    ActiveTab::NewLog
                } else {
                    ActiveTab::ViewLogs
                }
            }
            ActiveTab::ViewLogs => {
                if reverse {
                    ActiveTab::SpectrumView
                } else {
                    ActiveTab::NewLog
                }
            }
        };
    }

    async fn submit_form(&self, base_url: &str) {
        let frequency = self.frequency_input.borrow().trim().to_string();
        let grid_square = self.grid_square_input.borrow().trim().to_string();
        let callsign = self.callsign_input.borrow().trim().to_string();
        let comment = self.comment_input.borrow().trim().to_string();
        let duration_str = self.duration_input.borrow().trim().to_string();

        let frequency: f32 = match frequency.parse() {
            Ok(freq) => freq,
            Err(_) => {
                return;
            }
        };

        let duration: f32 = match duration_str.parse() {
            Ok(dur) => dur,
            Err(_) => {
                //TODO: auto-clear here
                *self.status_message.borrow_mut() = Some("✗ Invalid duration".to_string());
                return;
            }
        };

        let mut form = self.form_data.borrow_mut();
        form.frequency = frequency;
        form.grid_square = grid_square;
        form.callsign = callsign;
        form.comment = comment;
        form.recording_duration = duration;
        let url = format!("{}/logs", base_url);

        match api_client::create_log_async(&url, form.clone()).await {
            Ok(_) => {
                //TODO: auto-clear here
                *self.status_message.borrow_mut() = Some("✓ Submission successful".to_string());
                self.clear_form();
            }
            Err(e) => {
                *self.status_message.borrow_mut() = Some(format!("✗ Submission failed: {}", e));
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
        *self.status_message.borrow_mut() = None;
    }
}
