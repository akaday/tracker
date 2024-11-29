use std::error;
use std::time::{Duration, Instant};

use crate::components::{
    object_information::ObjectInformation, satellites::Satellites, track_map::TrackMap,
};

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

/// Application.
pub struct App {
    /// Is the application running?
    pub running: bool,

    pub track_map: TrackMap,
    pub object_information: ObjectInformation,
    pub satellites: Satellites,

    last_object_update: Instant,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            track_map: TrackMap::default(),
            object_information: ObjectInformation::default(),
            satellites: Satellites::default(),
            last_object_update: Instant::now(),
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
        if now.duration_since(self.last_object_update) >= OBJECT_UPDATE_INTERVAL {
            self.satellites.update_objects();
            self.last_object_update = now;
        }
    }

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }
}
