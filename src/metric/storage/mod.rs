pub mod builtin;

use serde::{Deserialize, Serialize};

use crate::metric::{MetricValue, storage::builtin::JsonStorageProviderConfig};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum StorageProviderConfig {
    Json(JsonStorageProviderConfig),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StorageConfig {
    pub provider: StorageProviderConfig,
}

pub trait StorageProvider {
    type Error: std::error::Error;
    async fn write(&self, id: i64, value: MetricValue) ->  Result<(), Self::Error>;
    async fn read(&self, id: &str, timestamp: i64) -> Result<Option<super::Metric>, Self::Error>;
}
