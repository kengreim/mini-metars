use serde::{Deserialize, Serialize};
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};
use tauri::utils::platform::current_exe;
use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;

#[derive(Debug, Serialize, Deserialize)]
struct Profile {
    pub stations: Vec<String>,
}

struct ProfileManager {
    profiles_dir_path: Option<PathBuf>,
}

impl ProfileManager {
    fn new() -> Self {
        Self {
            profiles_dir_path: dirs::config_local_dir().map(|mut p| {
                p.push("Mini METARs");
                p.push("Profiles");
                p
            }),
        }
    }
}

fn profiles_path() -> Option<PathBuf> {
    dirs::config_local_dir().map(|mut p| {
        p.push("Mini METARs");
        p.push("Profiles");
        p
    })
}

#[tauri::command]
async fn load_profile(app: AppHandle) {
    let exec_path = std::env::current_exe().unwrap().parent();
    //
    //
    // let default_path = profiles_path().map_or_else(|| current_exe())
    //
    // if let Some(path) = profiles_path() {
    //
    //
    //
    //     if let Ok(false) = path.try_exists() {
    //         create_dir_all(path);
    //     }
    // }

    let path = app
        .dialog()
        .file()
        .add_filter("Profile JSON", &[".json"])
        .blocking_pick_file();
}
