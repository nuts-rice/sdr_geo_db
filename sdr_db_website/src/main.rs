use std::{cell::RefCell, io, rc::Rc};

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Stylize, Modifier, Style },
    text::{Line, Span},
    widgets::{Block, BorderType, Paragraph},
    Frame, Terminal,
};

use ratzilla::{
    event::{KeyCode, KeyEvent, },
    DomBackend, WebRenderer,
};

use sdr_db::{LogFormData, SignalMode};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FormField {
    Frequency,
    Latitude,
    Longitude,
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
            FormField::Frequency => "Frequency (MHz):",
            FormField::Latitude => "Latitude:",
            FormField::Longitude => "Longitude:",
            FormField::Callsign => "Callsign:",
            FormField::Mode => "Mode:",
            FormField::Comment => "Comment:",
            FormField::RecordingDuration => "Recording Duration (seconds):",
        }
    }

    fn next(&self) -> Self {
        match self {
            FormField::Frequency => FormField::Latitude,
            FormField::Latitude => FormField::Longitude,
            FormField::Longitude => FormField::Callsign,
            FormField::Callsign => FormField::Mode,
            FormField::Mode => FormField::Comment,
            FormField::Comment => FormField::RecordingDuration,
            FormField::RecordingDuration => FormField::Frequency,
        }
    }

    fn previous(&self) -> Self {
        match self {
            FormField::Frequency => FormField::RecordingDuration,
            FormField::Latitude => FormField::Frequency,
            FormField::Longitude => FormField::Latitude,
            FormField::Callsign => FormField::Longitude,
            FormField::Mode => FormField::Callsign,
            FormField::Comment => FormField::Mode,
            FormField::RecordingDuration => FormField::Comment,
        }
    }

}



fn main() -> io::Result<()> {
    let backend = DomBackend::new()?;
    let terminal = Terminal::new(backend)?;

    let state = Rc::new(App::default());

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

#[derive(Default)]
struct App {
    form_data: RefCell<LogFormData>,
    frequency_input: RefCell<String>,
    latitude_input: RefCell<String>,
    longitude_input: RefCell<String>,
    callsign_input: RefCell<String>,
    mode_input: RefCell<SignalMode>,
    comment_input: RefCell<String>,
    duration_input: RefCell<String>,
    selected_field: RefCell<FormField>,
    state_message: RefCell<Option<String>>,
}

impl App {
    fn render(&self, frame: &mut Frame) {
        todo!()
    }

    fn handle_events(&self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Tab => {
                let mut selected = self.selected_field.borrow_mut();
                *selected =  if key_event.shift          {
                    selected.previous()
                } else {
                    selected.next()
                };

            }
            KeyCode::Enter => {
                self.submit_form();
            }
            _ => {}
        }
    }

    fn submit_form(&self) {
        todo!()
    }
}
