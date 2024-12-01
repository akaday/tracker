use std::io;

use anyhow::Result;
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::{
    app::App,
    event::{Event, EventHandler},
    handler::{handle_key_events, handle_mouse_events},
    tui::Tui,
};

pub mod app;
pub mod event;
pub mod handler;
pub mod object;
pub mod satellite;
pub mod tui;
pub mod widgets;

#[tokio::main]
async fn main() -> Result<()> {
    // Create an application.
    let mut app = App::new();

    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    // Start the main loop.
    while app.running {
        // Render the user interface.
        tui.render(&mut app)?;
        // Handle events.
        match tui.events.next().await? {
            Event::Tick => app.tick(),
            Event::Key(event) => handle_key_events(event, &mut app).await?,
            Event::Mouse(event) => handle_mouse_events(event, &mut app).await?,
            _ => {}
        }
    }

    // Exit the user interface.
    tui.exit()?;
    Ok(())
}
