use std::io;

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent};
use ratatui::{backend::CrosstermBackend, Terminal};
use widgets::{object_information, satellites, track_map};

use crate::{
    app::App,
    event::{Event, EventHandler},
    tui::Tui,
};

pub mod app;
pub mod event;
pub mod object;
pub mod satellite;
pub mod tui;
pub mod widgets;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new();
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    // Create an application.
    let mut app = App::new();

    // Start the main loop.
    while app.running {
        // Handle events.
        match tui.events.next().await? {
            Event::Update => app.update(),
            Event::Render => tui.render(&mut app)?,
            Event::Key(event) => handle_key_events(event, &mut app).await?,
            Event::Mouse(event) => handle_mouse_events(event, &mut app).await?,
        }
    }

    // Exit the user interface.
    tui.exit()?;
    Ok(())
}

async fn handle_key_events(event: KeyEvent, app: &mut App) -> Result<()> {
    match event.code {
        // Exit application on `ESC`
        KeyCode::Esc => {
            app.quit();
        }
        // Exit application on `Ctrl-C`
        KeyCode::Char('c') => {
            if event.modifiers == KeyModifiers::CONTROL {
                app.quit();
            }
        }
        _ => {}
    }
    Ok(())
}

async fn handle_mouse_events(event: MouseEvent, app: &mut App) -> Result<()> {
    track_map::handle_mouse_events(event, app).await?;
    object_information::handle_mouse_events(event, app).await?;
    satellites::handle_mouse_events(event, app).await?;
    Ok(())
}
