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

use ratatui::prelude::Rect;

use gloo_timers::future::TimeoutFuture;
use sdr_db::{LogFormData, SignalMode};
use wasm_bindgen_futures::spawn_local;
use web_sys::window;

pub mod api_client;
pub mod components;
pub mod spectrum_client;
pub mod tabs;

use spectrum_client::SpectrumClient;
use tabs::spectrum_view::{render_spectrum_view, SpectrumViewerState};
use tabs::view_logs::{create_header, create_table, ViewLogsState};

use components::geolocate_gridsquare;

const FONT: &str = "Hack Nerd Font";

const AMBER_BRIGHT: Color = Color::Rgb(255, 170, 0); // #FFAA00
const AMBER_MID: Color = Color::Rgb(170, 102, 0); // #AA6600
const MOAB_RED: Color = Color::Rgb(77, 37, 26); // #c55f42
const MOAB_PRIMARY_TABLE_ENTRY: Color = Color::Rgb(80, 42, 20); // #cd6c35
const MOAB_SECONDARY_TABLE_ENTRY: Color = Color::Rgb(102, 51, 25); // #d9784a
const MOAB_BACKGROUND: Color = Color::Rgb(54, 28, 22); // #361c16

const CATPPUCIN_YELLOW: Color = Color::Rgb(238, 212, 159); // #EED49F
const CATPPUCIN_ORANGE: Color = Color::Rgb(245, 153, 160); // #f5a97f
const CATPPUCIN_BLUE: Color = Color::Rgb(138, 173, 244); // #8aadf4
const CATPPUCIN_PRIMARY_TABLE_ENTRY: Color = Color::Rgb(68, 71, 90); // #44475a
const CATPPUCIN_SECONDARY_TABLE_ENTRY: Color = Color::Rgb(80, 82, 100); // #505264
const CATPPUCIN_BACKGROUND: Color = Color::Rgb(40, 42, 54); // #282a36

#[derive(Debug, Clone, Default)]
pub enum Theme {
    #[default]
    Moab,
    Catppuccin,
}

impl Theme {
    pub fn as_str(&self) -> &str {
        match self {
            Theme::Moab => "Moab",
            Theme::Catppuccin => "Catppuccin",
        }
    }

    pub fn primary_color(&self) -> Color {
        match self {
            Theme::Moab => AMBER_BRIGHT,
            Theme::Catppuccin => CATPPUCIN_YELLOW,
        }
    }

    pub fn secondary_color(&self) -> Color {
        match self {
            Theme::Moab => AMBER_MID,
            Theme::Catppuccin => CATPPUCIN_ORANGE,
        }
    }

    pub fn accent_color(&self) -> Color {
        match self {
            Theme::Moab => MOAB_RED,
            Theme::Catppuccin => CATPPUCIN_BLUE,
        }
    }

    pub fn primary_table_entry_color(&self) -> Color {
        match self {
            Theme::Moab => MOAB_PRIMARY_TABLE_ENTRY,
            Theme::Catppuccin => CATPPUCIN_PRIMARY_TABLE_ENTRY,
        }
    }

    pub fn secondary_table_entry_color(&self) -> Color {
        match self {
            Theme::Moab => MOAB_SECONDARY_TABLE_ENTRY,
            Theme::Catppuccin => CATPPUCIN_SECONDARY_TABLE_ENTRY,
        }
    }

    pub fn background_color(&self) -> Color {
        match self {
            Theme::Moab => MOAB_BACKGROUND,
            Theme::Catppuccin => CATPPUCIN_BACKGROUND,
        }
    }
}

#[derive(Clone, Copy)]
enum ActiveTab {
    NewLog,
    SpectrumView,
    ViewLogs,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
enum FormField {
    #[default]
    Frequency,
    GridSquare,
    Callsign,
    Mode,
    Comment,
    RecordingDuration,
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
    theme: RefCell<Theme>,
    theme_popup_visible: RefCell<bool>,
    theme_selected_index: RefCell<usize>,
    spectrum_data: RefCell<Vec<f64>>,
    spectrum_client: RefCell<Option<SpectrumClient>>,
    spectrum_view_state: RefCell<SpectrumViewerState>,
    spectrum_connected: RefCell<bool>,
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
            theme: RefCell::new(Theme::Moab),
            theme_popup_visible: RefCell::new(false),
            theme_selected_index: RefCell::new(0),
            spectrum_data: RefCell::new(Vec::new()),
            spectrum_client: RefCell::new(None),
            spectrum_view_state: RefCell::new(SpectrumViewerState::default()),
            spectrum_connected: RefCell::new(false),
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
        let tabs = ["New Log", "Spectrum View", "View Logs"];
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
            .highlight_style(
                Style::default()
                    .fg(self.theme.borrow().primary_color())
                    .add_modifier(Modifier::BOLD),
            )
            .divider(Span::raw("|"));
        frame.render_widget(tabs_widget, chunks[0]);

        let intro_popup_area = ratatui::layout::Rect {
            x: frame.area().width / 6,
            y: frame.area().height / 6,
            width: frame.area().width * 2 / 3,
            height: frame.area().height * 2 / 3,
        };
        if self.status_message.borrow().is_none()
            && self.frequency_input.borrow().is_empty()
            && self.grid_square_input.borrow().is_empty()
            && self.callsign_input.borrow().is_empty()
            && self.comment_input.borrow().is_empty()
            && self.duration_input.borrow().is_empty()
        {
            Self::render_intro_popup(frame, intro_popup_area);
            return;
        }

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
            .fg(self.theme.borrow().primary_color())
            .add_modifier(Modifier::BOLD);
        frame.render_widget(title, chunks[1]);

        // Content (switches based on active tab)
        match *self.active_tab.borrow() {
            ActiveTab::NewLog => {
                self.render_form(frame, chunks[2]);
            }
            ActiveTab::SpectrumView => {
                let state = self.spectrum_view_state.borrow();
                let connected = self.spectrum_connected.borrow();
                let theme = self.theme.borrow();
                render_spectrum_view(&state, frame, chunks[2], &theme, *connected);
            }
            ActiveTab::ViewLogs => {
                let theme = self.theme.borrow();
                self.render_logs_table(frame, chunks[2], &theme);
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
                let connected = *self.spectrum_connected.borrow();
                let state = self.spectrum_view_state.borrow();
                let msg = if connected {
                    format!(
                        "● Live | Center: {:.3} MHz | Span: {:.1} MHz",
                        state.center_frequency / 1_000_000.0,
                        state.span / 1_000_000.0
                    )
                } else {
                    "○ Connecting to spectrum source...".to_string()
                };
                let color = if connected { Color::Green } else { Color::Yellow };
                ("Spectrum", msg, color)
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
            ActiveTab::SpectrumView => "←/→: Change Tab | Live spectrum from SDR",
            ActiveTab::ViewLogs => "↑/↓: Scroll | R: Refresh | ←/→: Tab | Enter: Details",
        };
        let help = Line::from(vec![
            Span::raw(help_text),
            Span::raw("  |                                                               Made with 🦀 💜 🦀 in Colorado"),
        ]);
        frame.render_widget(help, chunks[4]);
        if *self.theme_popup_visible.borrow() {
            self.render_theme_selector_popup(frame, frame.area());
        }
    }

    fn render_theme_selector_popup(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        use ratatui::widgets::*;
        let popup_width = 50.min(area.width.saturating_sub(4));
        let popup_height = 7.min(area.height.saturating_sub(2));
        let popup_area = Rect {
            x: area.x + (area.width.saturating_sub(popup_width)) / 2,
            y: area.y + (area.height.saturating_sub(popup_height)) / 2,
            width: popup_width,
            height: popup_height,
        };

        frame.render_widget(Clear, popup_area);
        let selected = *self.theme_selected_index.borrow();
        let themes = [Theme::Moab, Theme::Catppuccin];
        let items: Vec<ListItem> = themes
            .iter()
            .enumerate()
            .map(|(idx, theme)| {
                let style = if idx == selected {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                let cursor = if idx == selected { ">" } else { " " };
                let line = format!("{} {}", cursor, theme.as_str());
                ListItem::new(line).style(style)
            })
            .collect();
        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Select Theme (↑↓ Enter Esc)")
                    .border_type(BorderType::Rounded),
            )
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

        frame.render_widget(list, popup_area);
    }

    fn render_intro_popup(frame: &mut Frame, area: ratatui::layout::Rect) {
        use ratatui::widgets::*;

        let text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "Welcome to the SDR Database!",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("Use the tabs above to navigate between different sections."),
            Line::from("In 'New Log', fill out the form to submit a new SDR contact entry."),
            Line::from("In 'Spectrum View', tune into live spectrum display. "),
            Line::from("In 'View Logs', browse and manage existing log entries."),
            Line::from(""),
            Line::from("Press any key to continue..."),
        ];

        let paragraph = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title("Intro"))
            .alignment(Alignment::Center);

        frame.render_widget(paragraph, area);
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

    fn on_theme_change(&self, new_theme: Theme) {
        *self.theme.borrow_mut() = new_theme;
    }

    fn render_logs_table(&self, frame: &mut Frame, area: ratatui::layout::Rect, theme: &Theme) {
        use ratatui::widgets::*;

        let state = self.view_log_state.borrow();

        if state.logs.is_empty() {
            let text = vec![
                Line::from(""),
                Line::from("No logs loaded."),
                Line::from(""),
                Line::from(Span::styled(
                    "Press 'R' to refresh",
                    Style::default().fg(theme.primary_color()),
                )),
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
            "https://opeth.taila716a3.ts.net".to_string()
        }
    }

    fn handle_events(&self, key_event: KeyEvent, app_rc: &Rc<Self>) {
        // Handle theme popup first (modal)
        if *self.theme_popup_visible.borrow() {
            match key_event.code {
                KeyCode::Up => {
                    let mut idx = self.theme_selected_index.borrow_mut();
                    *idx = idx.saturating_sub(1);
                }
                KeyCode::Down => {
                    let mut idx = self.theme_selected_index.borrow_mut();
                    *idx = (*idx + 1).min(1); // 2 themes: 0 and 1
                }
                KeyCode::Enter => {
                    let idx = *self.theme_selected_index.borrow();
                    let new_theme = match idx {
                        0 => Theme::Moab,
                        _ => Theme::Catppuccin,
                    };
                    *self.theme.borrow_mut() = new_theme;
                    *self.theme_popup_visible.borrow_mut() = false;
                }
                KeyCode::Esc => {
                    *self.theme_popup_visible.borrow_mut() = false;
                }
                _ => {}
            }
            return; // Don't process other keys when popup is open
        }

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
                app_rc.cycle_tabs(key_event.code == KeyCode::Left);
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
                't' => {
                    *self.theme_popup_visible.borrow_mut() = true;
                    *self.theme_selected_index.borrow_mut() = 0;
                }
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

    /*
        fn handle_active_tab_change(&self, new_tab: ActiveTab) {
            *self.active_tab.borrow_mut() = new_tab;
        }
    */
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

    fn cycle_tabs(self: &Rc<Self>, reverse: bool) {
        let new_tab = {
            let active_tab = self.active_tab.borrow();
            match *active_tab {
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
            }
        };

        *self.active_tab.borrow_mut() = new_tab;

        // Connect to spectrum WebSocket when switching to SpectrumView
        if matches!(new_tab, ActiveTab::SpectrumView) {
            self.connect_spectrum();
        }
    }

    fn connect_spectrum(self: &Rc<Self>) {
        // Skip if already connected
        if self.spectrum_client.borrow().is_some() {
            return;
        }

        let ws_url = Self::get_spectrum_ws_url();
        let app = Rc::clone(self);

        let client = SpectrumClient::connect(&ws_url, move |frame| {
            app.spectrum_view_state.borrow_mut().update_from_frame(frame);
            *app.spectrum_connected.borrow_mut() = true;
        });

        *self.spectrum_client.borrow_mut() = Some(client);
    }

    fn get_spectrum_ws_url() -> String {
        let window = window().expect("should have a window");
        let location = window.location();
        let hostname = location.hostname().unwrap_or_default();

        if hostname.contains("localhost") || hostname.starts_with("127.0.0.1") {
            "ws://localhost:3000/spectrum".to_string()
        } else {
            "wss://opeth.taila716a3.ts.net/spectrum".to_string()
        }
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
        let form_clone = {
            let mut form = self.form_data.borrow_mut();
            form.frequency = frequency;
            form.grid_square = grid_square;
            form.callsign = callsign;
            form.comment = comment;
            form.recording_duration = duration;
            form.clone()
        };
        let url = format!("{}/logs", base_url);

        match api_client::create_log_async(&url, form_clone).await {
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
