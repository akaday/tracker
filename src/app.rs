use std::error;
use std::time::{Duration, Instant};

use crate::widgets::{
    object_information::ObjectInformationState, satellites::SatellitesState,
    track_map::TrackMapState,
};

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

/// Application.
pub struct App {
    /// Is the application running?
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

    /// Handles the tick event of the terminal.
    pub fn tick(&mut self) {
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
