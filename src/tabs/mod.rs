pub mod create_log;
pub mod spectrum_view;
//pub mod view_logs;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Stylize, palette::tailwind},
    symbols,
    text::Line,
    widgets::{Block, Padding, Paragraph, Widget},
};
use strum::{Display, EnumIter, FromRepr};

const TAB_TITLE_PALETTE: [Color; 3] = [
    Color::Rgb(138, 173, 244),
    Color::Rgb(166, 218, 149),
    Color::Rgb(237, 135, 150),
];

#[derive(Default, Clone, Copy, Display, FromRepr, EnumIter, PartialEq)]
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
            Self::CreateLog => TAB_TITLE_PALETTE[0],
            Self::ViewLogs => TAB_TITLE_PALETTE[1],
            Self::SpectrumViewer => TAB_TITLE_PALETTE[2],
        }
    }
    pub fn block(self) -> Block<'static> {
        Block::bordered()
            .border_set(symbols::border::PROPORTIONAL_TALL)
            .padding(Padding::horizontal(1))
            .border_style(self.palette())
    }
    pub fn render_create_log_tab(
        self,
        form: &create_log::NewLogInputForm,
        area: Rect,
        buf: &mut Buffer,
    ) {
        create_log::render_create_log_form(form, area, buf);
    }
    pub fn render_view_logs_tab(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("View Logs")
            .block(self.block())
            .render(area, buf);
    }
    pub fn render_spectrum_viewer_tab(
        self,
        state: &spectrum_view::SpectrumViewerState,
        area: Rect,
        buf: &mut Buffer,
    ) {
        spectrum_view::render_spectrum_viewer(state, area, buf);
    }
}

impl Widget for SelectedTab {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self {
            SelectedTab::ViewLogs => {
                self.render_view_logs_tab(area, buf);
            }
            SelectedTab::CreateLog => {
                Paragraph::new("Create Log (use render_create_log_tab)")
                    .block(self.block())
                    .render(area, buf);
            }
            SelectedTab::SpectrumViewer => {
                Paragraph::new("Spectrum Viewer (use render_spectrum_viewer_tab)")
                    .block(self.block())
                    .render(area, buf);
            }
        }
    }
}
