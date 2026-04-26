use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::metric::{Metric, MetricValue};

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

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Generic storage error: {0}")]
    Generic(String),
}

pub trait StorageProviderConfig {}

#[async_trait]
pub trait StorageProvider {
    async fn write(&self, id: i64, timestamp: i64, value: MetricValue) -> Result<(), Error>;
    async fn read_timeline(
        &self,
        id: &str,
        timestamp: i64,
    ) -> Result<Option<Vec<(i64, MetricValue)>>, Error>;
    async fn read_all(&self, id: &str) -> Result<Option<Vec<(i64, MetricValue)>>, Error>;
}

pub async fn setup_provider(config: StorageConfig) -> Result<Box<dyn StorageProvider>, Error> {
    match config.provider {
        StorageConfigProvider::Json(json_config) => Ok(Box::new(
            builtin::JsonStorageProvider::init(json_config)
                .await
                .map_err(|e| Error::Generic(e.to_string()))?,
        )),
    }
}
