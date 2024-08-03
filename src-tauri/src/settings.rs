use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    pub show_vatsim_atis: bool,
    pub show_altimeter: bool,
    pub show_wind: bool,
}

impl Settings {
    pub fn new() -> Self {
        Settings {
            show_vatsim_atis: true,
            show_altimeter: true,
            show_wind: true,
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}

fn settings_path() -> Option<PathBuf> {
    dirs::config_local_dir().map(|p| p.join("Mini METARs"))
}
