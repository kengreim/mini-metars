#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

use crate::settings::{
    get_appstate_settings, get_latest_profile_path, read_settings_or_default,
    set_latest_profile_path,
};
use crate::window::{
    WindowState, apply_window_state, get_window_state, set_always_on_top_settings_checked,
};
use crate::{MAIN_WINDOW_LABEL, utils};
use log::debug;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager, PhysicalPosition, PhysicalSize, Wry};
use tauri_plugin_dialog::{DialogExt, FileDialogBuilder, FilePath};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub name: String,
    pub stations: Vec<String>,
    #[serde(default = "true_bool")]
    pub show_input: bool,
    #[serde(default = "true_bool")]
    pub show_titlebar: bool,
    pub window: Option<ProfileWindowState>,
    #[serde(default)]
    pub units: AltimeterUnits,
    #[serde(default = "false_bool")]
    pub hide_airport_if_missing_atis: bool,
}

const fn true_bool() -> bool {
    true
}

const fn false_bool() -> bool {
    false
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub enum AltimeterUnits {
    #[default]
    #[allow(non_camel_case_types)]
    inHg,
    #[allow(non_camel_case_types)]
    hPa,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileWindowState {
    pub state: WindowState,
    pub position: Option<PhysicalPosition<i32>>,
    pub size: Option<PhysicalSize<u32>>,
    #[serde(default = "default_scale")]
    pub scale_factor: f64,
}

pub const fn default_scale() -> f64 {
    1.0
}

fn profiles_path() -> Option<PathBuf> {
    dirs::config_local_dir().map(|p| p.join("Mini METARs").join("Profiles"))
}

fn get_or_create_profiles_path() -> Option<PathBuf> {
    profiles_path().and_then(|p| utils::get_or_create_path(&p))
}

pub fn read_profile_from_file(path: &Path) -> Result<Profile, anyhow::Error> {
    utils::deserialize_from_file(path)
}

fn write_profile_to_file(path: &Path, profile: &Profile) -> Result<(), anyhow::Error> {
    debug!("Writing profile to {path:?}");
    utils::serialize_to_file(path, profile)
}

fn profile_dialog_builder(app: &AppHandle) -> FileDialogBuilder<Wry> {
    let mut builder = app.dialog().file().add_filter("Profile JSON", &["json"]);

    let latest_profile = get_latest_profile_path(app);
    let latest_profile_dir = latest_profile
        .as_ref()
        .and_then(|p| p.parent().map(Path::to_path_buf));
    let latest_profile_filename = latest_profile.as_ref().and_then(|p| p.file_name());

    let dialog_path =
        latest_profile_dir.map_or_else(get_or_create_profiles_path, |p| match p.try_exists() {
            Ok(true) => Some(p),
            _ => None,
        });
    if dialog_path.is_some() {
        builder = builder.set_directory(dialog_path.unwrap());
    }

    if latest_profile_filename.is_some() {
        builder = builder.set_file_name(
            latest_profile_filename
                .unwrap()
                .to_os_string()
                .into_string()
                .unwrap_or_default(),
        );
    }

    builder
}

#[tauri::command(async)]
pub fn load_profile(app: AppHandle) -> Result<Profile, String> {
    debug!("Starting Load Profile Command");
    let window = app.get_webview_window(MAIN_WINDOW_LABEL);
    let settings = get_appstate_settings(&app).unwrap_or_else(read_settings_or_default);
    set_always_on_top_settings_checked(window.as_ref(), &settings, false)?;

    let pick_response = profile_dialog_builder(&app).blocking_pick_file();
    let ret = pick_response.map_or_else(
        || Err("Could not pick file".to_string()),
        |path| match path {
            FilePath::Url(_) => Err("Cannot load from URL".to_string()),
            FilePath::Path(path) => load_profile_from_path(&app, &path),
        },
    );

    set_always_on_top_settings_checked(window.as_ref(), &settings, true)?;

    ret
}

pub fn load_profile_from_path(app: &AppHandle, path: &PathBuf) -> Result<Profile, String> {
    debug!("Starting to load profile from: {path:?}");
    match read_profile_from_file(path) {
        Ok(profile) => {
            debug!("Found good profile");
            set_latest_profile_path(app, path);
            if let Some(window) = &profile.window {
                apply_window_state(app, window).map_err(|e| e.to_string())?;
            }
            Ok(profile)
        }
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command(async)]
pub fn save_current_profile(mut profile: Profile, app: AppHandle) -> Result<(), String> {
    debug!("Starting Save Current Profile Command");
    profile.window = get_window_state(&app);
    let last_profile_path = get_latest_profile_path(&app);
    if let Some(path) = last_profile_path {
        save_profile(&profile, &path, &app)
    } else {
        save_profile_as(profile, app)
    }
}

#[tauri::command(async)]
pub fn save_profile_as(mut profile: Profile, app: AppHandle) -> Result<(), String> {
    debug!("Starting Save Current Profile As Command");
    profile.window = get_window_state(&app);
    let window = app.get_webview_window(MAIN_WINDOW_LABEL);
    let settings = get_appstate_settings(&app).unwrap_or_else(read_settings_or_default);
    set_always_on_top_settings_checked(window.as_ref(), &settings, false)?;

    let ret = profile_dialog_builder(&app)
        .blocking_save_file()
        .map_or_else(
            || Err("Dialog closed without selecting save path".to_string()),
            |path| match path {
                FilePath::Url(_) => Err("Cannot save to URL".to_string()),
                FilePath::Path(path) => save_profile(&profile, &path, &app),
            },
        );

    set_always_on_top_settings_checked(window.as_ref(), &settings, true)?;

    ret
}

fn save_profile(profile: &Profile, path: &PathBuf, app: &AppHandle) -> Result<(), String> {
    debug!("Trying to write profile to {path:?}");
    match write_profile_to_file(path, profile) {
        Ok(()) => {
            debug!("Successfully wrote profile: {profile:?}");
            set_latest_profile_path(app, path);
            Ok(())
        }
        Err(e) => {
            debug!("Error writing profile: {e:?}");
            Err(e.to_string())
        }
    }
}
