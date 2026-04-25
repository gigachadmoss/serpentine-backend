pub mod builtin;

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::metric::storage::StorageConfig;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    pub storage: StorageConfig,
}

pub trait ConfigProvider {
    type Error: std::error::Error;
    fn get(&self) -> impl std::future::Future<Output = Result<Config, Self::Error>> + Send;
}

impl Default for Config {
    fn default() -> Self {
        Self {
            storage: StorageConfig {
                provider: crate::metric::storage::StorageProviderConfig::Json(
                    crate::metric::storage::builtin::JsonStorageProviderConfig {
                        path: std::path::PathBuf::from("metrics.json"),
                        flush_on_write: true,
                    },
                ),
            },
        }
    }
}

pub async fn setup_config_provider(path: PathBuf) -> Result<impl ConfigProvider, Box<dyn std::error::Error>> {
    builtin::json::JsonConfigProvider::init(path)
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}
