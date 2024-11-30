use anyhow::Result;
use chrono::{Duration, Utc};
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Margin, Position, Rect},
    style::{Color, Stylize},
    widgets::{
        canvas::{Canvas, Line, Map, MapResolution},
        Block, StatefulWidget, Widget,
    },
};

use crate::app::App;

use super::satellites::SatellitesState;

pub struct TrackMap<'a> {
    pub satellites_state: &'a SatellitesState,
    pub satellit_symbol: String,
    pub trajectory_color: Color,
}

#[derive(Default)]
pub struct TrackMapState {
    pub selected_object: Option<usize>,
    pub area: Rect,
}

impl StatefulWidget for TrackMap<'_> {
    type State = TrackMapState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.area = area;

        let bottom_layer = Canvas::default()
            .block(Block::bordered().title("Satellite ground track".blue()))
            .paint(|ctx| {
                // Draw the world map
                ctx.draw(&Map {
                    color: Color::Gray,
                    resolution: MapResolution::High,
                });

                // Draw each satellite's current position
                for object in self.satellites_state.objects.iter() {
                    let line = if state.selected_object.is_none() {
                        self.satellit_symbol.clone().light_red()
                            + format!(" {}", object.name()).white()
                    } else {
                        self.satellit_symbol.clone().red()
                            + format!(" {}", object.name()).dark_gray()
                    };
                    let state = object.predict(Utc::now()).unwrap();
                    ctx.print(state.position[0], state.position[1], line);
                }
            })
            .x_bounds([-180.0, 180.0])
            .y_bounds([-90.0, 90.0]);

        let top_layer = Canvas::default()
            .paint(|ctx| {
                if let Some(selected_object_index) = state.selected_object {
                    let selected = &self.satellites_state.objects[selected_object_index];
                    let state = selected.predict(Utc::now()).unwrap();

                    // Calculate future positions along the trajectory
                    let mut points = Vec::new();
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
                            ctx.draw(&Line::new(x1, y1, x_edge, y2, self.trajectory_color));
                            ctx.draw(&Line::new(-x_edge, y1, x2, y2, self.trajectory_color));
                            continue;
                        }
                        ctx.draw(&Line::new(x1, y1, x2, y2, self.trajectory_color));
                    }

                    // Highlight the selected satellite's current position
                    ctx.print(
                        state.position[0],
                        state.position[1],
                        self.satellit_symbol.clone().light_green().slow_blink()
                            + format!(" {}", selected.name()).white(),
                    );
                }
            })
            .x_bounds([-180.0, 180.0])
            .y_bounds([-90.0, 90.0]);

        bottom_layer.render(area, buf);
        top_layer.render(area.inner(Margin::new(1, 1)), buf);
    }
}

pub fn handle_mouse_events(event: MouseEvent, app: &mut App) -> Result<()> {
    let area = app.track_map_state.area;
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
                if let Some((index, _)) = app
                    .satellites_state
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
                    app.track_map_state.selected_object = Some(index);
                }
            }
            MouseButton::Right => {
                app.track_map_state.selected_object = None;
            }
            _ => {}
        }
    }

    Ok(())
}
