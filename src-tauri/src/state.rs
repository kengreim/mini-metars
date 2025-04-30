#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

use crate::awc::AviationWeatherCenterApi;
use crate::settings::Settings;
use crate::vatis::{AtisType, AtisUpdateMessage, AtisUpdateValue};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::OnceCell;
use vatsim_utils::errors::VatsimUtilError;
use vatsim_utils::live_api::Vatsim;
use vatsim_utils::models::V3ResponseData;

pub struct ExpiringEntry<T> {
    pub expiry: Instant,
    pub value: T,
}

impl<T> ExpiringEntry<T> {
    pub fn new_with_duration(value: T, duration: Duration) -> Self {
        Self {
            expiry: Instant::now() + duration,
            value,
        }
    }

    pub fn is_expired(&self) -> bool {
        Instant::now() > self.expiry
    }
}

pub type VatisCache = HashMap<(String, AtisType), ExpiringEntry<AtisUpdateMessage>>;

pub struct AppState {
    awc_client: OnceCell<Result<AviationWeatherCenterApi, anyhow::Error>>,
    vatsim_client: OnceCell<Result<Vatsim, VatsimUtilError>>,
    pub latest_vatsim_data: RwLock<Option<V3ResponseData>>,
    pub vatis_data: RwLock<Option<VatisCache>>,
    pub settings: RwLock<Option<Settings>>,
}

impl AppState {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            awc_client: OnceCell::const_new(),
            vatsim_client: OnceCell::const_new(),
            latest_vatsim_data: RwLock::new(None),
            vatis_data: RwLock::new(None),
            settings: RwLock::new(None),
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

pub fn get_cached_vatis_update<'a, 'b>(
    key: &'a (String, AtisType),
    opt: Option<&'b VatisCache>,
) -> Option<&'b AtisUpdateValue> {
    opt.and_then(|map| {
        map.get(key).map_or_else(
            || None,
            |entry| {
                if entry.is_expired() {
                    None
                } else {
                    Some(&entry.value.value)
                }
            },
        )
    })
}
