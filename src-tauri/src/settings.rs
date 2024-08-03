use crate::utils;
use crate::utils::deserialize_from_file;
use anyhow::anyhow;
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
    dirs::config_local_dir().map(|p| p.join("Mini METARs").join("settings.json"))
}

fn read_settings_or_default() -> Settings {
    settings_path().map_or_else(
        || Settings::default(),
        |p| deserialize_from_file(&p).map_or_else(|_| Settings::default(), |de| de),
    )
}

fn write_settings_to_file(settings: &Settings) -> Result<(), anyhow::Error> {
    settings_path().map_or_else(
        || Err(anyhow!("Could not construct path to settings.json")),
        |p| utils::serialize_to_file(&p, settings),
    )
}

// TODO -- get settings command for frontend

// TODO -- save settings command for frontend
