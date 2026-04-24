mod builtin;

use serde::{Deserialize, Serialize};

use crate::metric::storage::StorageConfig;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub storage: StorageConfig,
}

pub trait ConfigProvider {
    type Error: std::error::Error;
    async fn get(&self) -> Result<Config, Self::Error>;
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
