use crate::LockedState;
use serde::{Deserialize, Serialize};
use std::fs::{create_dir_all, File};
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager, Wry};
use tauri_plugin_dialog::{DialogExt, FileDialogBuilder};

#[derive(Debug, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub stations: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProfileResponse {
    pub filename: String,
    pub directory: String,
    pub data: Profile,
}

fn profiles_path() -> Option<PathBuf> {
    dirs::config_local_dir().map(|p| p.join("Mini METARs").join("Profiles"))
}

fn get_or_create_profiles_path() -> Option<PathBuf> {
    let result = profiles_path().map(|p| match p.try_exists() {
        Ok(true) => Some(p),
        Ok(false) => create_dir_all(&p).map_or(None, |()| Some(p)),
        _ => None,
    });

    result.unwrap_or_default()
}

fn read_profile_from_file(path: &Path) -> Result<Profile, anyhow::Error> {
    let file = File::open(path)?;
    let de = serde_json::from_reader(BufReader::new(file))?;
    Ok(de)
}

fn write_profile_to_file(path: &Path, profile: &Profile) -> Result<(), anyhow::Error> {
    let file = File::create(path)?;
    serde_json::to_writer_pretty(BufWriter::new(file), profile)?;
    Ok(())
}

async fn profile_dialog_builder(app: &AppHandle) -> FileDialogBuilder<Wry> {
    let mut builder = app.dialog().file().add_filter("Profile JSON", &["json"]);

    let latest_profile = get_latest_profile_path(app).await;
    let latest_profile_dir = latest_profile.as_ref().and_then(|p| p.parent().map(Path::to_path_buf));
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

async fn set_latest_profile_path(app: &AppHandle, path: PathBuf) {
    if let Some(state) = app.try_state::<LockedState>() {
        let mut path_state = state.last_profile_path.lock().await;
        *path_state = Some(path);
    }
}

async fn get_latest_profile_path(app: &AppHandle) -> Option<PathBuf> {
    if let Some(state) = app.try_state::<LockedState>() {
        let p = state.last_profile_path.lock().await;
        p.clone()
    } else {
        None
    }
}

#[tauri::command]
pub async fn load_profile(app: AppHandle) -> Result<Profile, String> {
    let pick_response = profile_dialog_builder(&app)
        .await
        .blocking_pick_file();

    if let Some(pick) = pick_response {
        match read_profile_from_file(&pick.path) {
            Ok(profile) => {
                set_latest_profile_path(&app, pick.path).await;
                Ok(profile)
            },
            Err(e) => Err(e.to_string())
        }
    } else {
        Err("Could not pick file".to_string())
    }
}

#[tauri::command]
pub async fn save_profile(profile: Profile, app: AppHandle) -> Result<(), String> {
    let save_path = profile_dialog_builder(&app)
        .await
        .blocking_save_file();

    if let Some(path) = save_path {
        match write_profile_to_file(&path, &profile) {
            Ok(()) => {
                set_latest_profile_path(&app, path).await;
                Ok(())
            },
            Err(e) => Err(e.to_string())
        }
    } else {
        Err("Could not select file to save".to_string())
    }
}
