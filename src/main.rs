use anyhow::Result;

use crate::app::App;

pub mod app;
pub mod event;
pub mod object;
pub mod satellite;
pub mod tui;
pub mod widgets;

#[tokio::main]
async fn main() -> Result<()> {
    App::new()?.run().await
}
