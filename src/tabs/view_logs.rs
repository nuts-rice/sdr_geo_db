use crate::Log;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style, Stylize},
    widgets::{Block, Borders, Cell, Row, Table, Widget},
};

// Theme colors
const HEADER_BG: Color = Color::Rgb(54, 68, 96);
const HEADER_FG: Color = Color::Rgb(14, 15, 23);
const SELECTED_STYLE_FG: Color = Color::Rgb(138, 173, 244);
const NORMAL_ROW_COLOR: Color = Color::Rgb(14, 15, 23);
const ALT_ROW_COLOR: Color = Color::Rgb(20, 21, 29);

// Column widths
const COL_WIDTH_ID: u16 = 5;
const COL_WIDTH_FREQUENCY: u16 = 12;
const COL_WIDTH_LAT: u16 = 10;
const COL_WIDTH_LON: u16 = 10;
const COL_WIDTH_CALLSIGN: u16 = 12;
const COL_WIDTH_MODE: u16 = 6;
const COL_WIDTH_DURATION: u16 = 10;
const COL_WIDTH_TIMESTAMP: u16 = 16;

// Format strings
const DATE_FORMAT: &str = "%Y-%m-%d %H:%M";
const PLACEHOLDER_CALLSIGN: &str = "N/A";

/// Table theme configuration for consistent styling
struct TableTheme {
    header_bg: Color,
    header_fg: Color,
    selected_fg: Color,
    normal_row: Color,
    alt_row: Color,
}

impl Default for TableTheme {
    fn default() -> Self {
        Self {
            header_bg: HEADER_BG,
            header_fg: HEADER_FG,
            selected_fg: SELECTED_STYLE_FG,
            normal_row: NORMAL_ROW_COLOR,
            alt_row: ALT_ROW_COLOR,
        }
    }
}

/// Extension trait for Log to provide table formatting
trait LogTableFormatter {
    fn to_table_row(&self) -> [String; 8];
}

impl LogTableFormatter for Log {
    fn to_table_row(&self) -> [String; 8] {
        [
            self.id.to_string(),
            format!("{:.2} MHz", self.frequency),
            format!("{:.4}°", self.xcoord),
            format!("{:.4}°", self.ycoord),
            self.callsign
                .as_deref()
                .unwrap_or(PLACEHOLDER_CALLSIGN)
                .to_string(),
            self.mode.clone(),
            format!("{:.1}s", self.recording_duration),
            self.timestamp.format(DATE_FORMAT).to_string(),
        ]
    }
}

#[derive(Debug, Default)]
pub struct ViewLogsState {
    pub logs: Vec<Log>,
    pub selected_index: usize,
    // TODO: Implement scrolling to utilize this field
    pub scroll_offset: usize,
}

impl ViewLogsState {
    pub fn new(logs: Vec<Log>) -> Self {
        Self {
            logs,
            selected_index: 0,
            scroll_offset: 0,
        }
    }

    /// Move selection to the next item
    pub fn select_next(&mut self) {
        if self.logs.is_empty() {
            return;
        }
        self.selected_index = (self.selected_index + 1).min(self.logs.len() - 1);
    }

    /// Move selection to the previous item
    pub fn select_previous(&mut self) {
        if self.logs.is_empty() {
            return;
        }
        self.selected_index = self.selected_index.saturating_sub(1);
    }

    /// Jump forward by page_size items
    pub fn select_next_page(&mut self, page_size: usize) {
        if self.logs.is_empty() {
            return;
        }
        self.selected_index = (self.selected_index + page_size).min(self.logs.len() - 1);
    }

    /// Jump backward by page_size items
    pub fn select_previous_page(&mut self, page_size: usize) {
        if self.logs.is_empty() {
            return;
        }
        self.selected_index = self.selected_index.saturating_sub(page_size);
    }

    /// Move selection to the first item
    pub fn select_first(&mut self) {
        if !self.logs.is_empty() {
            self.selected_index = 0;
        }
    }

    /// Move selection to the last item
    pub fn select_last(&mut self) {
        if !self.logs.is_empty() {
            self.selected_index = self.logs.len() - 1;
        }
    }

    /// Refresh the logs list and reset selection
    pub fn refresh_logs(&mut self, logs: Vec<Log>) {
        self.logs = logs;
        self.selected_index = 0;
        self.scroll_offset = 0;
    }

    /// Get the currently selected log, if any
    pub fn selected_log(&self) -> Option<&Log> {
        self.logs.get(self.selected_index)
    }
}

/// Create the table header
fn create_header(theme: &TableTheme) -> Row {
    let header_style = Style::default().fg(theme.header_fg).bg(theme.header_bg);

    ["ID", "Frequency", "Lat", "Lon", "Callsign", "Mode", "Duration", "Timestamp"]
        .into_iter()
        .map(Cell::from)
        .collect::<Row>()
        .style(header_style)
        .height(1)
}

/// Create a styled row for a log entry
fn create_row(log: &Log, index: usize, is_selected: bool, theme: &TableTheme) -> Row {
    let bg_color = if index % 2 == 0 {
        theme.normal_row
    } else {
        theme.alt_row
    };

    let mut row = Row::new(log.to_table_row()).style(Style::new().bg(bg_color));

    if is_selected {
        row = row.style(
            Style::default()
                .fg(theme.selected_fg)
                .add_modifier(Modifier::BOLD),
        );
    }

    row
}

/// Create all rows for the table
fn create_rows<'a>(
    state: &'a ViewLogsState,
    theme: &'a TableTheme,
) -> impl Iterator<Item = Row> + 'a {
    state
        .logs
        .iter()
        .enumerate()
        .map(|(i, log)| create_row(log, i, i == state.selected_index, theme))
}

/// Create the complete table widget
fn create_table(header: Row, rows: impl Iterator<Item = Row>, theme: &TableTheme) -> Table {
    let widths = [
        Constraint::Length(COL_WIDTH_ID),
        Constraint::Length(COL_WIDTH_FREQUENCY),
        Constraint::Length(COL_WIDTH_LAT),
        Constraint::Length(COL_WIDTH_LON),
        Constraint::Length(COL_WIDTH_CALLSIGN),
        Constraint::Length(COL_WIDTH_MODE),
        Constraint::Length(COL_WIDTH_DURATION),
        Constraint::Min(COL_WIDTH_TIMESTAMP),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .title("View Logs")
        .style(Style::default().bg(theme.normal_row));

    Table::new(rows, widths)
        .header(header)
        .block(block)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
}

/// Render the view logs table
pub fn render_view_logs(state: &ViewLogsState, area: Rect, buf: &mut Buffer) {
    let theme = TableTheme::default();

    let header = create_header(&theme);
    let rows = create_rows(state, &theme);
    let table = create_table(header, rows, &theme);

    table.render(area, buf);
}
