use crate::Theme;
use ratatui::{
    layout::Constraint,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
};
use sdr_db::api_types::LogResponse;

const COL_WIDTH_ID: u16 = 5;
const COL_WIDTH_FREQUENCY: u16 = 12;
const COL_WIDTH_CALLSIGN: u16 = 12;
const COL_WIDTH_MODE: u16 = 6;
const COL_WIDTH_DURATION: u16 = 10;
const COL_WIDTH_TIMESTAMP: u16 = 16;

#[derive(Debug, Default)]
pub struct ViewLogsState {
    pub logs: Vec<LogResponse>,
    pub selected_index: usize,
    // TODO: Implement scrolling to utilize this field
    pub scroll_offset: usize,
    pub logs_loaded: bool,
}

impl ViewLogsState {
    pub fn new() -> Self {
        Self {
            logs: Vec::new(),
            selected_index: 0,
            scroll_offset: 0,
            logs_loaded: false,
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
    pub fn refresh_logs(&mut self, logs: Vec<LogResponse>) {
        self.logs = logs;
        self.selected_index = 0;
        self.scroll_offset = 0;
    }

    /// Get the currently selected log, if any
    pub fn selected_log(&self) -> Option<&LogResponse> {
        self.logs.get(self.selected_index)
    }
}
pub fn create_header(theme: &Theme) -> Row<'static> {
    let header_style = Style::default()
        .fg(theme.primary_color())
        .bg(theme.secondary_color());

    [
        "ID",
        "Frequency",
        "Grid",
        "Callsign",
        "Mode",
        "Comment",
        "Timestamp",
        "Duration",
    ]
    .into_iter()
    .map(Cell::from)
    .collect::<Row>()
    .style(header_style)
    .height(1)
}

fn create_row(log: &LogResponse, index: usize, is_selected: bool, theme: &Theme) -> Row<'static> {
    let bg_color = if index % 2 == 0 {
        theme.primary_color()
    } else {
        theme.secondary_color()
    };

    // Use to_table_row() which handles all formatting + Option unwrapping
    let cells: Vec<Cell> = log.to_table_row().into_iter().map(Cell::from).collect();

    let mut row = Row::new(cells).style(Style::default().bg(bg_color));

    if is_selected {
        row = row.style(
            Style::default()
                .fg(theme.accent_color())
                .add_modifier(Modifier::BOLD),
        );
    }

    row
}

pub fn create_rows(state: &ViewLogsState, theme: &Theme) -> Vec<Row<'static>> {
    state
        .logs
        .iter()
        .enumerate()
        .map(|(i, log)| create_row(log, i, i == state.selected_index, theme))
        .collect()
}

/// Create the complete table widget
pub fn create_table<'a>(header: Row<'a>, rows: Vec<Row<'a>>, theme: &Theme) -> Table<'a> {
    let widths = [
        Constraint::Length(COL_WIDTH_ID),
        Constraint::Length(COL_WIDTH_FREQUENCY),
        Constraint::Length(COL_WIDTH_CALLSIGN),
        Constraint::Length(COL_WIDTH_MODE),
        Constraint::Length(COL_WIDTH_DURATION),
        Constraint::Min(COL_WIDTH_TIMESTAMP),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .title("View Logs")
        .style(Style::default().bg(theme.primary_color()).fg(Color::White));

    Table::new(rows, widths)
        .header(header)
        .block(block)
        .row_highlight_style(Style::default().add_modifier(Modifier::BOLD))
}

/*
pub fn handle_scroll_state(direction: ScrollDirection) {
    todo!()
}
*/
