use std::cell::Cell;

use anyhow::Result;
use chrono::{Duration, Utc};
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::{
    layout::{Margin, Position, Rect},
    style::{Color, Stylize},
    symbols::Marker,
    widgets::{
        canvas::{Canvas, Line, Map, MapResolution},
        Block,
    },
    Frame,
};

use crate::app::App;

use super::Component;

#[derive(Default)]
pub struct TrackMap {
    pub selected_object: Option<usize>,
    area: Cell<Rect>,
}

impl TrackMap {
    pub fn area(&self) -> Rect {
        self.area.get()
    }
}

impl Component for TrackMap {
    fn render(&self, app: &App, frame: &mut Frame, area: Rect) -> Result<()> {
        const SATELLITE_SYMBOL: &str = "+";

        self.area.set(area);

        let bottom_layer = Canvas::default()
            .block(Block::bordered().title("Satellite ground track".blue()))
            .marker(Marker::Braille)
            .paint(|ctx| {
                // Draw the world map
                ctx.draw(&Map {
                    color: Color::Gray,
                    resolution: MapResolution::High,
                });

                // Draw each satellite's current position
                for object in &app.satellites.objects {
                    let line = if self.selected_object.is_none() {
                        SATELLITE_SYMBOL.light_red() + format!(" {}", object.name()).white()
                    } else {
                        SATELLITE_SYMBOL.red() + format!(" {}", object.name()).dark_gray()
                    };
                    let state = object.predict(Utc::now()).unwrap();
                    ctx.print(state.position[0], state.position[1], line);
                }
            })
            .x_bounds([-180.0, 180.0])
            .y_bounds([-90.0, 90.0]);

        let top_layer = Canvas::default()
            .marker(Marker::Braille)
            .paint(|ctx| {
                if let Some(selected_object_index) = self.selected_object {
                    let selected = &app.satellites.objects[selected_object_index];
                    let state = selected.predict(Utc::now()).unwrap();

                    // Calculate future positions along the trajectory
                    let mut points = Vec::new();
                    points.push((state.position[0], state.position[1]));
                    for minutes in 1..selected.orbital_period().num_minutes() {
                        let time = Utc::now() + Duration::minutes(minutes);
                        let state = selected.predict(time).unwrap();
                        points.push((state.position[0], state.position[1]));
                    }

                    // Draw the lines between predicted points
                    for window in points.windows(2) {
                        let (x1, y1) = window[0];
                        let (x2, y2) = window[1];
                        // Handle trajectory crossing the international date line
                        if (x1 - x2).abs() >= 180.0 {
                            let x_edge = if x1 > 0.0 { 180.0 } else { -180.0 };
                            ctx.draw(&Line::new(x1, y1, x_edge, y2, Color::LightBlue));
                            ctx.draw(&Line::new(-x_edge, y1, x2, y2, Color::LightBlue));
                            continue;
                        }
                        assert!((y1 - y2).abs() < 90.0);
                        ctx.draw(&Line::new(x1, y1, x2, y2, Color::LightBlue));
                    }

                    // Highlight the selected satellite's current position
                    ctx.print(
                        state.position[0],
                        state.position[1],
                        SATELLITE_SYMBOL.light_green().rapid_blink()
                            + format!(" {}", selected.name()).white(),
                    );
                }
            })
            .x_bounds([-180.0, 180.0])
            .y_bounds([-90.0, 90.0]);

        frame.render_widget(bottom_layer, area);
        frame.render_widget(top_layer, area.inner(Margin::new(1, 1)));

        Ok(())
    }
}

pub fn handle_mouse_events(event: MouseEvent, app: &mut App) -> Result<()> {
    let area = app.track_map.area();
    if !area.contains(Position::new(event.column, event.row)) {
        return Ok(());
    }

    if let MouseEventKind::Down(buttom) = event.kind {
        match buttom {
            MouseButton::Left => {
                // Convert mouse coordinates to latitude and longitude
                let x = (event.column as f64 - area.left() as f64) / area.width as f64;
                let y = (event.row as f64 - area.top() as f64) / area.height as f64;
                let lon = -180.0 + x * 360.0;
                let lat = 90.0 - y * 180.0;

                // Find the nearest object
                if let Some((index, _)) =
                    app.satellites
                        .objects
                        .iter()
                        .enumerate()
                        .min_by_key(|(_, obj)| {
                            let state = obj.predict(Utc::now()).unwrap();
                            let dx = state.longitude() - lon;
                            let dy = state.latitude() - lat;
                            ((dx * dx + dy * dy) * 1000.0) as i32
                        })
                {
                    app.track_map.selected_object = Some(index);
                }
            }
            MouseButton::Right => {
                app.track_map.selected_object = None;
            }
            _ => {}
        }
    }

    Ok(())
}
