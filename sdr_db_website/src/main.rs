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
pub mod app;
pub mod components;
pub mod spectrum_client;
pub mod tabs;
pub mod theme;

use crate::app::{get_base_url, App};
use components::geolocate_gridsquare;
use spectrum_client::SpectrumClient;
use tabs::spectrum_view::{render_spectrum_view, SpectrumViewerState};
use tabs::view_logs::{create_header, create_table, ViewLogsState};
use theme::Theme;

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
    //TODO: hook up ws logic to this
    let state = Rc::new(App::new(get_base_url("http")));

    let event_state = Rc::clone(&state);
    terminal.on_key_event(move |key_event| {
        event_state.handle_events(key_event);
    });

    let render_state = Rc::clone(&state);
    terminal.draw_web(move |frame| {
        render_state.render(frame);
    });

    Ok(())
}
