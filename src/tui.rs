use std::{io, panic};

use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Layout},
    style::Color,
    Terminal,
};

use crate::{
    app::App,
    event::EventHandler,
    widgets::{object_information::ObjectInformation, satellites::Satellites, track_map::TrackMap},
};

/// Representation of a terminal user interface.
///
/// It is responsible for setting up the terminal,
/// initializing the interface and handling the draw events.
#[derive(Debug)]
pub struct Tui<B: Backend> {
    /// Interface to the Terminal.
    terminal: Terminal<B>,
    /// Terminal event handler.
    pub events: EventHandler,
}

impl<B: Backend> Tui<B> {
    /// Constructs a new instance of [`Tui`].
    pub fn new(terminal: Terminal<B>, events: EventHandler) -> Self {
        Self { terminal, events }
    }

    /// Initializes the terminal interface.
    ///
    /// It enables the raw mode and sets terminal properties.
    pub fn init(&mut self) -> Result<()> {
        terminal::enable_raw_mode()?;
        crossterm::execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;

        // Define a custom panic hook to reset the terminal properties.
        // This way, you won't have your terminal messed up if an unexpected error happens.
        let panic_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic| {
            Self::reset().expect("failed to reset the terminal");
            panic_hook(panic);
        }));

        self.terminal.hide_cursor()?;
        self.terminal.clear()?;
        Ok(())
    }

    /// [`Draw`] the terminal interface by [`rendering`] the widgets.
    ///
    /// [`Draw`]: ratatui::Terminal::draw
    /// [`rendering`]: crate::ui::render
    pub fn render(&mut self, app: &mut App) -> Result<()> {
        self.terminal.draw(|frame| {
            let horizontal = Layout::horizontal([Constraint::Percentage(80), Constraint::Min(25)]);
            let [left, right] = horizontal.areas(frame.area());
            let vertical = Layout::vertical([Constraint::Percentage(60), Constraint::Fill(1)]);
            let [top_right, bottom_right] = vertical.areas(right);

            frame.render_stateful_widget(Satellites, bottom_right, &mut app.satellites_state);

            let track_map = TrackMap {
                satellites_state: &app.satellites_state,
                satellit_symbol: "+".to_string(),
                trajectory_color: Color::LightBlue,
            };
            frame.render_stateful_widget(track_map, left, &mut app.track_map_state);

            let object_information = ObjectInformation {
                satellites_state: &app.satellites_state,
                track_map_state: &app.track_map_state,
            };
            frame.render_stateful_widget(
                object_information,
                top_right,
                &mut app.object_information_state,
            );
        })?;
        Ok(())
    }

    /// Resets the terminal interface.
    ///
    /// This function is also used for the panic hook to revert
    /// the terminal properties if unexpected errors occur.
    fn reset() -> Result<()> {
        terminal::disable_raw_mode()?;
        crossterm::execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
        Ok(())
    }

    /// Exits the terminal interface.
    ///
    /// It disables the raw mode and reverts back the terminal properties.
    pub fn exit(&mut self) -> Result<()> {
        Self::reset()?;
        self.terminal.show_cursor()?;
        Ok(())
    }
}
