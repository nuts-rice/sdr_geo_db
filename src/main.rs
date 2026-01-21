use diesel::{Connection, PgConnection};
use ratatui::buffer::Buffer;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Borders, Paragraph};
use sdr_db::Log;
use sdr_db::create_log;
use sdr_db::model::log::render_log;
use sdr_db::tabs::{SelectedTab, create_log::NewLogInputForm, spectrum_view::SpectrumViewerState};

use clap::Parser;
use tracing::{error, info};

use color_eyre::Result;
use crossterm::event::{Event, KeyCode};
use ratatui::{
    DefaultTerminal,
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    text::{Line, Text},
    widgets::{Block, Tabs, Widget},
};

use strum::IntoEnumIterator;

#[derive(Parser, Debug)]
#[command(name = "sdr_db")]
#[command(about = "SDR Database - Collect and store SDR measurements with geospatial data", long_about = None)]
struct Args {
    /// Database URL (or use DATABASE_URL env var)
    #[arg(long, env = "DATABASE_URL")]
    database_url: Option<String>,

    /// Latitude in decimal degrees [-90, 90]
    #[arg(long)]
    latitude: Option<f32>,

    /// Longitude in decimal degrees [-180, 180]
    #[arg(long)]
    longitude: Option<f32>,

    /// Frequency in Hz (must be positive)
    #[arg(long)]
    frequency: Option<f32>,

    /// Callsign or station identifier
    #[arg(long)]
    callsign: Option<String>,

    /// Mode (e.g., FM, AM, SSB)
    #[arg(long)]
    mode: String,

    /// Optional comment
    #[arg(long)]
    comment: Option<String>,

    #[arg(long)]
    recording_duration: Option<f32>,
}

struct App {
    state: AppState,
    selected_tab: SelectedTab,
    new_log_form: NewLogInputForm,
    spectrum_viewer_state: SpectrumViewerState,
}

#[derive(Default, Clone, Eq, PartialEq)]
pub enum AppState {
    #[default]
    Running,
    Quitting,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: AppState::Running,
            selected_tab: SelectedTab::CreateLog,
            new_log_form: NewLogInputForm::default(),
            spectrum_viewer_state: SpectrumViewerState::default(),
        }
    }
    pub fn run(mut self, mut terminal: DefaultTerminal, database_url: &str) -> Result<()> {
        while self.state == AppState::Running {
            let conn = &mut PgConnection::establish(database_url)?;

            terminal.draw(|frame| frame.render_widget(&self, frame.area()))?;
            self.handle_events(conn)?;
        }
        Ok(())
    }

    fn render_tabs(&self, area: Rect, buf: &mut Buffer) {
        let titles = SelectedTab::iter().map(SelectedTab::title);
        let highlight_style = (Color::default(), self.selected_tab.palette());
        let selected_tab_idx = self.selected_tab as usize;
        Tabs::new(titles)
            .highlight_style(highlight_style)
            .select(selected_tab_idx)
            .padding("", "")
            .divider(" ")
            .render(area, buf);
    }

    fn submit_log_entry(&mut self, conn: &mut PgConnection) {
        let form = &self.new_log_form;

        match create_log(
            conn,
            form.frequency,
            form.grid_square.clone(),
            form.callsign.to_string(),
            form.mode,
            form.comment.clone(),
            form.recording_duration,
        ) {
            Ok(log) => {
                info!("✓ Log entry created successfully!");
                self.new_log_form.created_log = Some(log);
            }
            Err(e) => {
                error!("Failed to create log entry: {}", e);
            }
        }
    }

    fn handle_events(&mut self, conn: &mut PgConnection) -> std::io::Result<()> {
        // Use poll with timeout for continuous animation on spectrum viewer
        let timeout = if self.selected_tab == SelectedTab::SpectrumViewer {
            std::time::Duration::from_millis(50) // 20 FPS animation
        } else {
            std::time::Duration::from_millis(100)
        };

        if crossterm::event::poll(timeout)?
            && let Event::Key(key) = crossterm::event::read()?
            && key.kind == crossterm::event::KeyEventKind::Press
        {
            // If popup is showing, any key dismisses it
            if self.new_log_form.created_log.is_some() {
                self.new_log_form = NewLogInputForm::default();
                return Ok(());
            }

            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => {
                    //       if self.selected_tab != SelectedTab::CreateLog =>
                    //   {
                    self.quit();
                    return Ok(());
                }
                KeyCode::Char('l') => {
                    self.next_tab();
                    return Ok(());
                }
                KeyCode::Char('h') => {
                    self.previous_tab();
                    return Ok(());
                }
                _ => {}
            }
            match &self.selected_tab {
                SelectedTab::CreateLog => match key.code {
                    KeyCode::Enter => {
                        self.submit_log_entry(conn);
                    }
                    KeyCode::Esc => {
                        self.new_log_form = NewLogInputForm::default();
                    }
                    KeyCode::Tab => {
                        if key
                            .modifiers
                            .contains(crossterm::event::KeyModifiers::SHIFT)
                        {
                            self.new_log_form.previous_field();
                        } else {
                            self.new_log_form.next_field();
                        }
                    }
                    _ => {
                        self.new_log_form.handle_key_event(key);
                    }
                },
                SelectedTab::SpectrumViewer => {
                    if self.spectrum_viewer_state.is_file_popup_visible() {
                        match key.code {
                            KeyCode::Esc => {
                                self.spectrum_viewer_state.hide_file_source_popup();
                            }
                            KeyCode::Enter => {
                                let _ = self.spectrum_viewer_state.confirm_file_source();
                            }
                            KeyCode::Backspace => {
                                self.spectrum_viewer_state.file_popup_backspace();
                            }
                            KeyCode::Left => {
                                self.spectrum_viewer_state.file_popup_cursor_left();
                            }
                            KeyCode::Right => {
                                self.spectrum_viewer_state.file_popup_cursor_right();
                            }
                            KeyCode::Char(c) => {
                                self.spectrum_viewer_state.file_popup_input(c);
                            }
                            _ => {}
                        }
                    } else {
                        match key.code {
                            KeyCode::Up => {
                                self.spectrum_viewer_state.increase_frequency();
                            }
                            KeyCode::Down => {
                                self.spectrum_viewer_state.decrease_frequency();
                            }
                            KeyCode::Tab => {
                                self.spectrum_viewer_state.toggle_source();
                            }
                            KeyCode::Char('f') => {
                                self.spectrum_viewer_state.show_file_source_popup();
                            }
                            KeyCode::Char('l') | KeyCode::Char('L') => {
                                if key
                                    .modifiers
                                    .contains(crossterm::event::KeyModifiers::SHIFT)
                                {
                                    self.spectrum_viewer_state.decrease_lna_gain();
                                } else {
                                    self.spectrum_viewer_state.increase_lna_gain();
                                }
                            }
                            KeyCode::Char('v') | KeyCode::Char('V') => {
                                if key
                                    .modifiers
                                    .contains(crossterm::event::KeyModifiers::SHIFT)
                                {
                                    self.spectrum_viewer_state.decrease_vga_gain();
                                } else {
                                    self.spectrum_viewer_state.increase_vga_gain();
                                }
                            }
                            _ => {}
                        }
                    }
                }
                _ => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        self.quit();
                    }
                    KeyCode::Char('l') => {
                        self.next_tab();
                    }
                    KeyCode::Char('h') => {
                        self.previous_tab();
                    }
                    _ => {}
                },
            }
        }

        // Update spectrum viewer animation when on that tab
        if self.selected_tab == SelectedTab::SpectrumViewer {
            self.spectrum_viewer_state.tick(0.05); // Advance time by 0.05 radians
        }

        Ok(())
    }

    pub fn next_tab(&mut self) {
        self.selected_tab = self.selected_tab.next();
    }

    pub fn previous_tab(&mut self) {
        self.selected_tab = self.selected_tab.previous();
    }

    pub fn quit(&mut self) {
        self.state = AppState::Quitting;
    }
}
fn render_title(buf: &mut Buffer, area: Rect) {
    "SDR DB".bold().render(area, buf);
}
fn render_footer(area: Rect, buf: &mut Buffer) {
    Line::raw("◄ ► to change tab | Press q to quit")
        .centered()
        .render(area, buf);
}
fn create_log_exit_popup(area: Rect, buf: &mut Buffer, log: &Log) {
    let popup_block = Block::default()
        .title("Log Created")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Rgb(14, 15, 23)));
    let exit_string = format!(
        "New entry created: {}\n Press any key to continue...",
        render_log(log)
    );
    let exit_text = Text::styled(exit_string, Style::default().fg(Color::Rgb(93, 53, 59)));

    let popup_area = Rect {
        x: area.x + area.width / 8,
        y: area.y + area.height / 4,
        width: area.width * 3 / 4,
        height: area.height / 2,
    };
    let popup_paragraph = Paragraph::new(exit_text).block(popup_block);
    popup_paragraph.render(popup_area, buf);
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        use Constraint::{Length, Min};
        let vertical = Layout::vertical([Length(1), Min(0), Length(1)]);
        let [header_area, inner_area, footer_area] = vertical.areas(area);

        let horizontal = Layout::horizontal([Min(0), Length(20)]);
        let [tabs_area, title_area] = horizontal.areas(header_area);

        render_title(buf, title_area);
        self.render_tabs(tabs_area, buf);
        match self.selected_tab {
            SelectedTab::CreateLog => {
                self.selected_tab
                    .render_create_log_tab(&self.new_log_form, inner_area, buf);
                if let Some(ref log) = self.new_log_form.created_log {
                    create_log_exit_popup(area, buf, log);
                }
            }
            SelectedTab::ViewLogs => {
                self.selected_tab.render_view_logs_tab(inner_area, buf);
            }
            SelectedTab::SpectrumViewer => {
                self.selected_tab.render_spectrum_viewer_tab(
                    &self.spectrum_viewer_state,
                    inner_area,
                    buf,
                );
            }
        }
        render_footer(footer_area, buf);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();
    let database_url = dotenvy::var("DATABASE_URL").expect("DATABASE_URL must be set ");
    // Initialize terminal
    let terminal = ratatui::init();

    let result = App::new().run(terminal, &database_url);
    ratatui::restore();

    result?;
    Ok(())
}
