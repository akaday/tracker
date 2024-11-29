use std::{fs, path::PathBuf, time::Duration};

use strum::{Display, EnumIter};
use ureq::serde_json;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Display, EnumIter)]
pub enum Satellite {
    // Space stations
    #[strum(to_string = "CSS")]
    Css,
    #[strum(to_string = "ISS")]
    Iss,

    // Weather satellites
    Weather,
    #[strum(to_string = "NOAA")]
    Noaa,
    #[strum(to_string = "GOES")]
    Goes,

    // Earth resources satellites
    #[strum(to_string = "Earth resources")]
    EarthResources,
    #[strum(to_string = "Search & rescue")]
    SearchRescue,
    #[strum(to_string = "Disaster monitoring")]
    DisasterMonitoring,

    // Navigation satellites
    #[strum(to_string = "GPS Operational")]
    Gps,
    #[strum(to_string = "GLONASS Operational")]
    Glonass,
    Galileo,
    Beidou,

    // Scientific satellites
    #[strum(to_string = "Space & Earth Science")]
    SpaceEarthScience,
    Geodetic,
    Engineering,
    Education,

    // Miscellaneous satellites
    #[strum(to_string = "DFH-1")]
    Dfh1,
    Military,
    #[strum(to_string = "Radar calibration")]
    RadarCalibration,
    CubeSats,
}

impl Satellite {
    pub fn get_elements(&self) -> Vec<sgp4::Elements> {
        let cache_path = PathBuf::from(format!("cache/{}.json", self.to_string().to_lowercase()));
        fs::create_dir_all(cache_path.parent().unwrap()).unwrap();

        let should_update = match fs::metadata(&cache_path) {
            Ok(metadata) => {
                metadata.modified().unwrap().elapsed().unwrap() > Duration::from_secs(2 * 60 * 60)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => true,
            _ => panic!(),
        };

        if should_update {
            let elements = self.fetch_elements();
            fs::write(&cache_path, serde_json::to_string(&elements).unwrap()).unwrap();
            elements
        } else {
            let json = fs::read_to_string(&cache_path).unwrap();
            serde_json::from_str(&json).unwrap()
        }
    }

    /// Returns the international designator
    fn cospar_id(&self) -> Option<&str> {
        match self {
            Self::Iss => Some("1998-067A"),
            Self::Css => Some("2021-035A"),
            Self::Dfh1 => Some("1970-034A"),
            _ => None,
        }
    }

    /// Returns CelesTrak group name
    fn group(&self) -> Option<&str> {
        match self {
            Self::Weather => Some("weather"),
            Self::Noaa => Some("noaa"),
            Self::Goes => Some("goes"),
            Self::EarthResources => Some("resource"),
            Self::SearchRescue => Some("sarsat"),
            Self::DisasterMonitoring => Some("dmc"),
            Self::Gps => Some("gps-ops"),
            Self::Glonass => Some("glo-ops"),
            Self::Galileo => Some("galileo"),
            Self::Beidou => Some("beidou"),
            Self::SpaceEarthScience => Some("science"),
            Self::Geodetic => Some("geodetic"),
            Self::Engineering => Some("engineering"),
            Self::Education => Some("education"),
            Self::Military => Some("military"),
            Self::RadarCalibration => Some("radar"),
            Self::CubeSats => Some("cubesat"),
            _ => None,
        }
    }

    fn fetch_elements(&self) -> Vec<sgp4::Elements> {
        let request = ureq::get("https://celestrak.org/NORAD/elements/gp.php");
        if let Some(id) = self.cospar_id() {
            let response = request
                .query("INTDES", id)
                .query("FORMAT", "json")
                .call()
                .unwrap();
            return response.into_json().unwrap();
        }
        if let Some(group) = self.group() {
            let response = request
                .query("GROUP", group)
                .query("FORMAT", "json")
                .call()
                .unwrap();
            return response.into_json().unwrap();
        }
        unreachable!();
    }
}
