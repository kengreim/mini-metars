#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::awc::{AviationWeatherCenterApi, MetarDto, Station};
use serde::{Deserialize, Serialize};
use tauri::State;
use tokio::sync::{OnceCell, RwLock};

mod awc;

pub struct AppState {
    awc_client: OnceCell<Result<AviationWeatherCenterApi, anyhow::Error>>
}

impl AppState {
    #[must_use]
    pub const fn new() -> Self {
        Self { awc_client: OnceCell::const_new() }
    }

    pub async fn get_awc_client(&self) -> &Result<AviationWeatherCenterApi, anyhow::Error> {
        self.awc_client.get_or_init(|| async {
            AviationWeatherCenterApi::try_new().await
        }).await
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
            fetch_metar,
            lookup_station
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
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
    if let Ok(client) = &state.0.read().await.get_awc_client().await {
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
    if let Ok(client) = &state.0.read().await.get_awc_client().await {
        client
            .lookup_station(id)
            .map_err(|e| format!("Error looking up station {id}: {e:?}"))
    } else {
        Err("AWC Api Client not initialized".to_string())
    }
}
