use std::time::{Duration, Instant};

use ratatui::{
    layout::{Constraint, Layout},
    style::Color,
    Frame,
};

use crate::widgets::{
    object_information::{ObjectInformation, ObjectInformationState},
    satellites::{Satellites, SatellitesState},
    track_map::{TrackMap, TrackMapState},
};

/// Application.
pub struct App {
    /// Indicates if the application is currently active and running. When set to false, triggers application shutdown.
    pub running: bool,

    pub track_map_state: TrackMapState,
    pub satellites_state: SatellitesState,
    pub object_information_state: ObjectInformationState,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            track_map_state: Default::default(),
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

        frame.render_stateful_widget(Satellites, bottom_right, &mut self.satellites_state);

        let track_map = TrackMap {
            satellites_state: &self.satellites_state,
            satellit_symbol: "+".to_string(),
            trajectory_color: Color::LightBlue,
        };
        frame.render_stateful_widget(track_map, left, &mut self.track_map_state);

        let object_information = ObjectInformation {
            satellites_state: &self.satellites_state,
            track_map_state: &self.track_map_state,
        };
        frame.render_stateful_widget(
            object_information,
            top_right,
            &mut self.object_information_state,
        );
    }

    /// Handles the tick event of the terminal.
    pub fn update(&mut self) {
        const OBJECT_UPDATE_INTERVAL: Duration = Duration::from_secs(2 * 60);
        let now = Instant::now();
        if now.duration_since(self.satellites_state.last_object_update) >= OBJECT_UPDATE_INTERVAL {
            self.satellites_state.update_objects();
            self.satellites_state.last_object_update = now;
        }
    }

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }
}
