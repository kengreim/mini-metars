#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

use crate::awc::AviationWeatherCenterApi;
use crate::settings::Settings;
use crate::vatis;
use cached::TimedCache;
use std::sync::Mutex;
use tokio::sync::OnceCell;
use vatsim_utils::errors::VatsimUtilError;
use vatsim_utils::live_api::Vatsim;
use vatsim_utils::models::V3ResponseData;

pub struct AppState {
    awc_client: OnceCell<Result<AviationWeatherCenterApi, anyhow::Error>>,
    vatsim_client: OnceCell<Result<Vatsim, VatsimUtilError>>,
    pub latest_vatsim_data: Mutex<Option<V3ResponseData>>,
    pub vatis_data: Mutex<Option<TimedCache<String, vatis::AtisUpdateMessage>>>,
    pub settings: Mutex<Option<Settings>>,
}

impl AppState {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            awc_client: OnceCell::const_new(),
            vatsim_client: OnceCell::const_new(),
            latest_vatsim_data: Mutex::new(None),
            vatis_data: Mutex::new(None),
            settings: Mutex::new(None),
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
