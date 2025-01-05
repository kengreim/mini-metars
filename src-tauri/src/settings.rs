#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

use crate::profiles::{load_profile_from_path, Profile};
use crate::state::AppState;
use crate::utils;
use crate::utils::deserialize_from_file;
use anyhow::anyhow;
use log::debug;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Manager};

const fn true_bool() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    #[serde(default = "true_bool")]
    load_most_recent_profile_on_open: bool,
    most_recent_profile: Option<PathBuf>,
    #[serde(default = "true_bool")]
    always_on_top: bool,
    #[serde(default = "true_bool")]
    auto_resize: bool,
}

impl Settings {
    pub const fn new() -> Self {
        Self {
            load_most_recent_profile_on_open: true,
            most_recent_profile: None,
            always_on_top: true,
            auto_resize: true,
        }
    }

    pub const fn always_on_top(&self) -> bool {
        self.always_on_top
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

pub fn read_settings_or_default() -> Settings {
    settings_path().map_or_else(
        || {
            debug!("Could not construct path to settings.json");
            Settings::default()
        },
        |p| match deserialize_from_file(&p) {
            Ok(s) => {
                debug!("Read settings from {p:?} - {s:?}");
                s
            }
            Err(e) => {
                debug!("Error reading settings: {e}");
                Settings::default()
            }
        },
    )
}

fn write_settings_to_file(settings: &Settings) -> Result<(), anyhow::Error> {
    debug!("Starting write settings to file: {:?}", settings);
    settings_path().map_or_else(
        || {
            const E: &str = "Could not construct path to settings.json";
            debug!("Error writing settings: {E}");
            Err(anyhow!(E))
        },
        |p| utils::serialize_to_file(&p, settings),
    )
}

pub fn set_appstate_settings(app: &AppHandle, settings: Settings) {
    if let Some(state) = app.try_state::<Arc<AppState>>() {
        debug!("Setting in-memory settings: {settings:?}");
        *state.settings.lock().unwrap() = Some(settings);
    } else {
        debug!("Could not get app state");
    }
}

pub fn get_appstate_settings(app: &AppHandle) -> Option<Settings> {
    let ret = app
        .try_state::<Arc<AppState>>()
        .and_then(|state| state.settings.lock().unwrap().clone());
    debug!("Retrieved in-memory settings: {ret:?}");

    ret
}

pub fn set_latest_profile_path(app: &AppHandle, path: &PathBuf) {
    if let Some(state) = app.try_state::<Arc<AppState>>() {
        let mut settings = state.settings.lock().unwrap();
        *settings = (*settings).as_ref().map_or_else(
            || Some(read_settings_or_default()),
            |s| {
                Some(Settings {
                    most_recent_profile: Some(path.clone()),
                    ..s.clone()
                })
            },
        );
        drop(settings);
        debug!("Set in-memory latest profile path: {path:?}");
    }
}

pub fn get_latest_profile_path(app: &AppHandle) -> Option<PathBuf> {
    let ret = app.try_state::<Arc<AppState>>().and_then(|state| {
        state
            .settings
            .lock()
            .unwrap()
            .as_ref()
            .and_then(|s| s.most_recent_profile.clone())
    });

    debug!("Retrieved in-memory latest profile path: {ret:?}");
    ret
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InitialSettingsLoadResponse {
    pub settings: Settings,
    pub profile: Option<Profile>,
}

#[tauri::command(async)]
pub fn load_settings_initial(app: AppHandle) -> Result<InitialSettingsLoadResponse, String> {
    debug!("Starting Load Settings Initial Command");

    let settings = get_appstate_settings(&app).unwrap_or_else(|| {
        let settings = read_settings_or_default();
        set_appstate_settings(&app, settings.clone());
        settings
    });

    let mut profile = None;
    if settings.load_most_recent_profile_on_open {
        if let Some(path) = settings.most_recent_profile.as_ref() {
            profile = Some(load_profile_from_path(&app, path)?);
        }
    }

    Ok(InitialSettingsLoadResponse { settings, profile })
}

#[tauri::command(async)]
pub fn load_settings(app: AppHandle) -> Settings {
    debug!("Starting Load Settings Command");
    let ret = read_settings_or_default();
    set_appstate_settings(&app, ret.clone());

    ret
}

#[tauri::command(async)]
pub fn save_settings(app: AppHandle, settings: Option<Settings>) -> Result<(), String> {
    debug!("Starting Save Settings Command");
    let appstate_settings = get_appstate_settings(&app).unwrap_or_else(read_settings_or_default);
    let write_settings = settings.map_or(appstate_settings.clone(), |s| Settings {
        most_recent_profile: appstate_settings.most_recent_profile,
        ..s
    });

    write_settings_to_file(&write_settings).map_err(|e| e.to_string())
}
