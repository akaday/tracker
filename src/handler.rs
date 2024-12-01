use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent};

use crate::{
    app::App,
    widgets::{object_information, satellites, track_map},
};

pub async fn handle_key_events(event: KeyEvent, app: &mut App) -> Result<()> {
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

pub async fn handle_mouse_events(event: MouseEvent, app: &mut App) -> Result<()> {
    track_map::handle_mouse_events(event, app).await?;
    object_information::handle_mouse_events(event, app).await?;
    satellites::handle_mouse_events(event, app).await?;
    Ok(())
}
