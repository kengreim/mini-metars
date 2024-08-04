use async_trait::async_trait;
use dashmap::DashMap;
use std::time::{Duration, Instant};
use vatsim_utils::live_api::Vatsim;
use vatsim_utils::models::V3ResponseData;

pub struct CachedApiClient<T>
where
    T: Clone + Send + Sync,
{
    client: Box<dyn ApiFetch<T> + Sync>,
    data: DashMap<String, DataRecord<T>>,
    ttl: Duration,
}

pub struct DataRecord<T>
where
    T: Clone,
{
    data: T,
    fetched_at: Instant,
    valid_until: Instant,
}

#[async_trait]
pub trait ApiFetch<T> {
    async fn fetch(&self, key: &str) -> Result<T, anyhow::Error>;
}

impl<T> CachedApiClient<T>
where
    T: Clone + Send + Sync,
{
    pub fn new_from(base_client: impl ApiFetch<T> + Sync + 'static, duration: Duration) -> Self {
        Self {
            client: Box::new(base_client),
            data: DashMap::new(),
            ttl: duration,
        }
    }

    pub async fn get(&self, key: &str) -> Result<T, anyhow::Error> {
        let cache_result: Option<T> = self.data.get(key).and_then(|ret| {
            if ret.value().valid_until.elapsed() < self.ttl {
                Some(ret.value().data.clone())
            } else {
                None
            }
        });

        if let Some(t) = cache_result {
            Ok(t)
        } else {
            let new_val = self.client.fetch(key).await?;
            let _ = self.insert(key, new_val.clone());
            Ok(new_val)
        }
    }

    fn insert(&self, key: &str, val: T) -> Option<DataRecord<T>> {
        self.data.insert(
            key.to_string(),
            DataRecord {
                data: val,
                fetched_at: Instant::now(),
                valid_until: Instant::now() + self.ttl,
            },
        )
    }
}

pub struct VatsimApiWrapper {
    pub client: Vatsim,
}

impl VatsimApiWrapper {
    pub async fn new() -> Result<Self, anyhow::Error> {
        let c = Vatsim::new().await?;
        Ok(Self { client: c })
    }
}

#[async_trait]
impl ApiFetch<V3ResponseData> for VatsimApiWrapper {
    async fn fetch(&self, _: &str) -> Result<V3ResponseData, anyhow::Error> {
        Ok(self.client.get_v3_data().await?)
    }
}

async fn test() {
    let x = Vatsim::new().await.unwrap();
    let y = VatsimApiWrapper { client: x };
    let z = CachedApiClient::new_from(y, Duration::from_secs(60));
}
