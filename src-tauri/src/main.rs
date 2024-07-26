#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::awc::{AviationWeatherCenterApi, MetarDto};
use tauri::State;
use tokio::sync::RwLock;

mod awc;

pub struct AppState {
    pub awc_client: Option<AviationWeatherCenterApi>,
}

impl AppState {
    #[must_use]
    pub const fn new() -> Self {
        Self { awc_client: None }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

pub struct LockedState(pub RwLock<AppState>);

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn main() {
    tauri::Builder::default()
        .manage(LockedState(RwLock::new(AppState::new())))
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![initialize_client, fetch_metar])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
async fn initialize_client(state: State<'_, LockedState>) -> Result<(), String> {
    initialize_client_private(&state).await
}

async fn initialize_client_private(state: &State<'_, LockedState>) -> Result<(), String> {
    match AviationWeatherCenterApi::try_new().await {
        Ok(client) => {
            state.0.write().await.awc_client = Some(client);
            Ok(())
        }
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
async fn fetch_metar(id: &str, state: State<'_, LockedState>) -> Result<MetarDto, String> {
    if state.0.read().await.awc_client.is_none() {
        initialize_client_private(&state).await?;
    }

    if let Some(client) = &state.0.read().await.awc_client {
        client
            .fetch_metar(id)
            .await
            .map_err(|e| format!("Error fetching METARs: {e:?}"))
    } else {
        Err("AWC Api Client not initialized".to_string())
    }
}
