use std::time::{Duration, Instant};

use ratatui::{
    layout::{Constraint, Layout},
    style::Color,
    Frame,
};

use crate::widgets::{
    object_information::{ObjectInformation, ObjectInformationState},
    satellites::{Satellites, SatellitesState},
    world_map::{WorldMap, WorldMapState},
};

/// Application.
pub struct App {
    /// Indicates if the application is currently active and running. When set to false, triggers application shutdown.
    pub running: bool,

    pub world_map_state: WorldMapState,
    pub satellites_state: SatellitesState,
    pub object_information_state: ObjectInformationState,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            world_map_state: Default::default(),
            satellites_state: Default::default(),
            object_information_state: Default::default(),
        }
    }
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        Self::default()
    }

    pub fn render(&mut self, frame: &mut Frame) {
        let horizontal = Layout::horizontal([Constraint::Percentage(80), Constraint::Min(25)]);
        let [left, right] = horizontal.areas(frame.area());
        let vertical = Layout::vertical([Constraint::Percentage(60), Constraint::Fill(1)]);
        let [top_right, bottom_right] = vertical.areas(right);

        let world_map = WorldMap {
            satellites_state: &self.satellites_state,
            satellit_symbol: "+".to_string(),
            trajectory_color: Color::LightBlue,
        };
        frame.render_stateful_widget(world_map, left, &mut self.world_map_state);

        let object_information = ObjectInformation {
            satellites_state: &self.satellites_state,
            world_map_state: &self.world_map_state,
        };
        frame.render_stateful_widget(
            object_information,
            top_right,
            &mut self.object_information_state,
        );

        frame.render_stateful_widget(Satellites, bottom_right, &mut self.satellites_state);
    }

    /// Handles the tick event of the terminal.
    pub async fn update(&mut self) {
        self.refresh_objects().await;
    }

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    async fn refresh_objects(&mut self) {
        const OBJECT_UPDATE_INTERVAL: Duration = Duration::from_secs(2 * 60);
        let now = Instant::now();
        if now.duration_since(self.satellites_state.last_object_update) >= OBJECT_UPDATE_INTERVAL {
            self.satellites_state.refresh_objects().await;
            self.satellites_state.last_object_update = now;
        }
    }
}
