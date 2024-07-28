#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::awc::{AviationWeatherCenterApi, MetarDto, Station};
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::State;
use tokio::sync::{Mutex, OnceCell};
use vatsim_utils::errors::VatsimUtilError;
use vatsim_utils::live_api::Vatsim;
use vatsim_utils::models::V3ResponseData;

mod awc;

pub struct VatsimDataFetch {
    pub fetched_time: Instant,
    pub data: Result<V3ResponseData, anyhow::Error>,
}

impl VatsimDataFetch {
    #[must_use]
    pub fn new(data: Result<V3ResponseData, anyhow::Error>) -> Self {
        Self {
            fetched_time: Instant::now(),
            data,
        }
    }
}

pub struct AppState {
    awc_client: OnceCell<Result<AviationWeatherCenterApi, anyhow::Error>>,
    vatsim_client: OnceCell<Result<Vatsim, VatsimUtilError>>,
    latest_vatsim_data: Mutex<Option<VatsimDataFetch>>,
}

impl AppState {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            awc_client: OnceCell::const_new(),
            vatsim_client: OnceCell::const_new(),
            latest_vatsim_data: Mutex::const_new(None),
        }
    }

    pub async fn get_awc_client(&self) -> &Result<AviationWeatherCenterApi, anyhow::Error> {
        self.awc_client
            .get_or_init(|| async { AviationWeatherCenterApi::try_new().await })
            .await
    }

    pub async fn get_vatsim_client(&self) -> &Result<Vatsim, VatsimUtilError> {
        self.vatsim_client
            .get_or_init(|| async { Vatsim::new().await })
            .await
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

type LockedState = Arc<AppState>;
//pub struct LockedState(pub Arc<RwLock<AppState>>);

fn main() {
    tauri::Builder::default()
        .manage(Arc::new(AppState::new()))
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            fetch_metar,
            lookup_station,
            get_atis_letter
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
    if let Ok(client) = &state.get_awc_client().await {
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
    if let Ok(client) = &state.get_awc_client().await {
        client
            .lookup_station(id)
            .map_err(|e| format!("Error looking up station {id}: {e:?}"))
    } else {
        Err("AWC Api Client not initialized".to_string())
    }
}

#[tauri::command]
async fn get_atis_letter(icao_id: &str, state: State<'_, LockedState>) -> Result<String, String> {
    let mut data = state.latest_vatsim_data.lock().await;

    if (*data).as_ref().map_or_else(|| true, datafeed_is_stale) {
        *data = Some(VatsimDataFetch::new(fetch_vatsim_data(&state).await));
    }

    if let Some(fetch) = &*data {
        fetch.data.as_ref().map_or_else(
            |_| Err("Could not retrieve datafeed".to_string()),
            |datafeed| {
                datafeed
                    .atis
                    .iter()
                    .find(|a| a.callsign.starts_with(icao_id))
                    .map_or_else(
                        || Ok("-".to_string()),
                        |a| a.clone().atis_code.map_or_else(|| Ok("-".to_string()), Ok),
                    )
            },
        )
    } else {
        Err("Could not retrieve datafeed".to_string())
    }
}

fn datafeed_is_stale(fetch: &VatsimDataFetch) -> bool {
    fetch.fetched_time.elapsed() > Duration::from_secs(60)
}

async fn fetch_vatsim_data(
    state: &State<'_, LockedState>,
) -> Result<V3ResponseData, anyhow::Error> {
    if let Ok(client) = state.get_vatsim_client().await {
        client.get_v3_data().await.map_err(Into::into)
    } else {
        Err(anyhow!("VATSIM API client not initialized".to_string()))
    }
}
