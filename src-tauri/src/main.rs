#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::awc::{AviationWeatherCenterApi, MetarDto, Station};
use serde::{Deserialize, Serialize};
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

fn main() {
    tauri::Builder::default()
        .manage(LockedState(RwLock::new(AppState::new())))
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            initialize_client,
            fetch_metar,
            lookup_station
        ])
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct FetchMetarResponse {
    metar: MetarDto,
    wind_string: String,
    altimeter: f64,
}

#[tauri::command]
async fn fetch_metar(
    id: &str,
    state: State<'_, LockedState>,
) -> Result<FetchMetarResponse, String> {
    if state.0.read().await.awc_client.is_none() {
        initialize_client_private(&state).await?;
    }

    if let Some(client) = &state.0.read().await.awc_client {
        client
            .fetch_metar(id)
            .await
            .map_err(|e| format!("Error fetching METARs: {e:?}"))
            .map(|m| FetchMetarResponse {
                wind_string: m.wind_string(),
                altimeter: m.altimeter_in_hg(),
                metar: m,
            })
    } else {
        Err("AWC Api Client not initialized".to_string())
    }
}

#[tauri::command]
async fn lookup_station(id: &str, state: State<'_, LockedState>) -> Result<Station, String> {
    if state.0.read().await.awc_client.is_none() {
        initialize_client_private(&state).await?;
    }

    if let Some(client) = &state.0.read().await.awc_client {
        client
            .lookup_station(id)
            .map_err(|e| format!("Error looking up station {id}: {e:?}"))
    } else {
        Err("AWC Api Client not initialized".to_string())
    }
}
