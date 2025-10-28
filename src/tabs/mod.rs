pub mod create_log;
//pub mod spectrum_view;
//pub mod view_logs;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Stylize, palette::tailwind},
    symbols,
    text::Line,
    widgets::{Block, Padding, Paragraph, Widget},
};
use strum::{Display, EnumIter, FromRepr, IntoEnumIterator};

use crate::create_log;

#[derive(Default, Clone, Copy, Display, FromRepr, EnumIter)]
pub enum SelectedTab {
    #[default]
    #[strum(to_string = "Create Log")]
    CreateLog,
    #[strum(to_string = "View Logs")]
    ViewLogs,
    #[strum(to_string = "Spectrum Viewer")]
    SpectrumViewer,
}

impl SelectedTab {
    pub fn previous(self) -> Self {
        let current_idx: usize = self as usize;
        let previous_idx = current_idx.saturating_sub(1);
        Self::from_repr(previous_idx).unwrap_or(self)
    }
    pub fn next(self) -> Self {
        let current_idx = self as usize;
        let next_idx = current_idx.saturating_add(1);
        Self::from_repr(next_idx).unwrap_or(self)
    }
    pub fn title(self) -> Line<'static> {
        format!("   {self}  ")
            .fg(tailwind::SLATE.c200)
            .bg(self.palette())
            .into()
    }

    pub fn palette(self) -> ratatui::style::Color {
        match self {
            Self::CreateLog => Color::Rgb(138, 173, 244),
            Self::ViewLogs => Color::Rgb(166, 218, 149),
            Self::SpectrumViewer => Color::Rgb(237, 135, 150),
        }
    }
    pub fn block(self) -> Block<'static> {
        Block::bordered()
            .border_set(symbols::border::PROPORTIONAL_TALL)
            .padding(Padding::horizontal(1))
            .border_style(self.palette())
    }
    pub fn render_create_log_tab(self, area: Rect, buf: &mut Buffer) {}
    pub fn render_view_logs_tab(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("View Logs")
            .block(self.block())
            .render(area, buf);
    }
    pub fn render_spectrum_viewer_tab(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("Look! I'm different than others!")
            .block(self.block())
            .render(area, buf);
    }
}

impl Widget for SelectedTab {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self {
            SelectedTab::ViewLogs => {
                self.render_view_logs_tab(area, buf);
            }
            SelectedTab::CreateLog => {
                self.render_create_log_tab(area, buf);
            }
            SelectedTab::SpectrumViewer => {
                self.render_spectrum_viewer_tab(area, buf);
            }
        }
    }
}
