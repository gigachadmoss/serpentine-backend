use serde::{Deserialize, Serialize};

use crate::metric::MetricValue;

pub mod builtin;

use builtin::JsonStorageProviderConfig;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum StorageConfigProvider {
    Json(JsonStorageProviderConfig),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct StorageConfig {
    pub provider: StorageConfigProvider,
}

trait StorageProviderConfig {}

pub trait StorageProvider {
    type Error: std::error::Error;
    type Config: StorageProviderConfig;

    async fn write(&self, id: i64, value: MetricValue) ->  Result<(), Self::Error>;
    async fn read(&self, id: &str, timestamp: i64) -> Result<Option<super::Metric>, Self::Error>;
}
