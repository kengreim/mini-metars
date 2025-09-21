use anyhow::{anyhow, bail};
use chrono::serde::ts_seconds;
use chrono::{DateTime, Utc};
use flate2::read::GzDecoder;
use log::debug;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;
use std::io::Read;

const BASE_URL: &str = "https://aviationweather.gov/";
const MBAR_TO_INHG_FACTOR: f64 = 0.02953;

#[derive(Clone)]
pub struct AviationWeatherCenterApi {
    client: Client,
    stations: Option<HashMap<String, Station>>,
    faa_icao_lookup: Option<HashMap<String, String>>,
}

impl AviationWeatherCenterApi {
    pub async fn try_new() -> Result<Self, anyhow::Error> {
        let mut new = Self {
            client: Client::builder().build()?,
            stations: None,
            faa_icao_lookup: None,
        };

        new.update_stations().await?;
        Ok(new)
    }

    fn metars_json_url(airports_string: &str) -> String {
        format!("{BASE_URL}/api/data/metar/?ids={airports_string}&format=json")
    }

    pub async fn fetch_metar(&self, station_id: &str) -> Result<MetarDto, anyhow::Error> {
        if station_id.starts_with('@') || station_id.len() > 4 || station_id.contains(',') {
            bail!("Invalid station ID, must be a single ICAO or FAA ID")
        }

        let id_sanitized = self.sanitize_id(station_id);

        let metars = self
            .client
            .get(Self::metars_json_url(&id_sanitized))
            .send()
            .await?
            .json::<Vec<MetarDto>>()
            .await?;

        if metars.is_empty() {
            Err(anyhow!("No METARs found in result list"))
        } else {
            Ok(metars[0].clone())
        }
    }

    #[allow(dead_code)]
    pub async fn fetch_metars(
        &self,
        station_ids: &[&str],
    ) -> Result<Vec<MetarDto>, reqwest::Error> {
        let sanitized_ids = station_ids
            .iter()
            .map(|id| self.sanitize_id(id))
            .collect::<Vec<_>>();

        self.client
            .get(Self::metars_json_url(&sanitized_ids.join(",")))
            .send()
            .await?
            .json::<Vec<MetarDto>>()
            .await
    }

    pub async fn update_stations(&mut self) -> Result<HashMap<String, Station>, anyhow::Error> {
        let stations = self.fetch_stations_hashmap().await?;
        self.stations = Some(stations.clone());
        self.faa_icao_lookup = Some(
            stations
                .values()
                .map(|s| (s.faa_id.to_uppercase(), s.icao_id.to_uppercase()))
                .collect(),
        );
        Ok(stations)
    }

    pub async fn fetch_stations(&self) -> Result<Vec<Station>, anyhow::Error> {
        let gzipped = self
            .client
            .get(format!("{BASE_URL}/data/cache/stations.cache.json.gz"))
            .send()
            .await?
            .bytes()
            .await?;

        // AWC doesn't set a header that reqwest automatically catches, so need
        // to do manual GZIP decompression
        let read = &gzipped.into_iter().collect::<Vec<_>>()[..];
        let mut d = GzDecoder::new(read);
        let mut s = String::new();
        d.read_to_string(&mut s)?;

        Ok(serde_json::from_str(&s)?)
    }

    pub async fn fetch_stations_hashmap(&self) -> Result<HashMap<String, Station>, anyhow::Error> {
        let stations = self.fetch_stations().await?;
        let map = stations
            .into_iter()
            .map(|s| (s.icao_id.to_uppercase(), s))
            .collect::<HashMap<_, _>>();
        Ok(map)
    }

    pub fn lookup_station(&self, lookup_id: &str) -> Result<Station, anyhow::Error> {
        let uppercase = lookup_id.to_uppercase();
        if let (Some(stations), Some(faa_icao_map)) = (&self.stations, &self.faa_icao_lookup) {
            if let Some(station) = stations.get(&uppercase) {
                Ok(station.clone())
            } else if let Some(id) = faa_icao_map.get(&uppercase) {
                stations.get(&id.to_uppercase()).map_or_else(
                    || Err(anyhow!("Error: inconsistency between FAA and ICAO data")),
                    |s| Ok(s.clone()),
                )
            } else {
                bail!("Error: could not find ID in ICAO or FAA lookups")
            }
        } else {
            bail!("Error: station data not initialized")
        }
    }

    fn sanitize_id(&self, id: &str) -> String {
        let ret = self
            .stations
            .as_ref()
            .map_or(id.to_uppercase(), |stations| {
                let id_is_state = id.starts_with('@') && id.len() == 3;
                let id_is_valid_icao = stations.contains_key(id);

                if id_is_state || id_is_valid_icao {
                    id.to_uppercase()
                } else if let Some(faa_icao_map) = &self.faa_icao_lookup {
                    faa_icao_map
                        .get(id)
                        .map_or_else(|| id.to_uppercase(), ToString::to_string)
                } else {
                    id.to_uppercase()
                }
            });
        debug!("Sanitized id for {id}: {ret}");

        ret
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Station {
    pub icao_id: String,
    pub iata_id: String,
    pub faa_id: String,
    pub wmo_id: String,
    pub lat: f64,
    pub lon: f64,
    pub elev: i32,
    pub site: String,
    pub state: String,
    pub country: String,
    pub priority: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetarDto {
    pub icao_id: String,
    pub receipt_time: String,
    #[serde(deserialize_with = "ts_seconds::deserialize")]
    pub obs_time: DateTime<Utc>,
    pub report_time: String,
    pub temp: Option<f64>,
    pub dewp: Option<f64>,
    pub wdir: Option<StringOrI32>,
    pub wspd: Option<i32>,
    pub wgst: Option<i32>,
    pub visib: StringOrF64,
    pub altim: f64,
    pub slp: Option<f64>,
    pub qc_field: i32,
    pub wx_string: Option<String>,
    pub pres_tend: Option<f64>,
    pub max_t: Option<f64>,
    pub min_t: Option<f64>,
    pub max_t24: Option<f64>,
    pub min_t24: Option<f64>,
    pub precip: Option<f64>,
    pub pcp3hr: Option<f64>,
    pub pcp6hr: Option<f64>,
    pub pcp24hr: Option<f64>,
    pub snow: Option<f64>,
    pub vert_vis: Option<i32>,
    pub metar_type: String,
    pub raw_ob: String,
    pub most_recent: Option<i32>,
    pub lat: f64,
    pub lon: f64,
    pub elev: i32,
    pub prior: Option<i32>,
    pub name: String,
    pub clouds: Vec<Cloud>,
}

impl MetarDto {
    pub fn altimeter_in_hg(&self) -> f64 {
        (self.altim * MBAR_TO_INHG_FACTOR * 100.0).round() / 100.0
    }

    pub const fn altimeter_hpa(&self) -> f64 {
        self.altim
    }

    pub fn wind_string(&self) -> String {
        if let (Some(wind_dir), Some(wind_spd)) = (&self.wdir, self.wspd) {
            let mut return_s = String::new();

            let dir_str = match wind_dir {
                StringOrI32::String(s) => s.to_string(),
                StringOrI32::I32(i) => format!("{i:03}"),
            };
            return_s.push_str(&dir_str);
            return_s.push_str(&format!("{wind_spd:02}"));
            if let Some(gusts) = self.wgst {
                return_s.push_str(&format!("G{gusts}"));
            }
            return_s.push_str("KT");

            return_s
        } else {
            String::new()
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cloud {
    pub cover: String,
    pub base: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StringOrI32 {
    String(String),
    I32(i32),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StringOrF64 {
    String(String),
    F64(f64),
}

impl fmt::Display for StringOrI32 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self {
            Self::String(s) => write!(f, "{s}"),
            Self::I32(i) => write!(f, "{i}"),
        }
    }
}

impl fmt::Display for StringOrF64 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self {
            Self::String(s) => write!(f, "{s}"),
            Self::F64(i) => write!(f, "{i}"),
        }
    }
}
