pub mod builtin;

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{interface::{InterfaceConfig, builtin::{HttpInterfaceProviderConfig, http::Listener}}, metric::storage::{StorageConfig, StorageConfigProvider, builtin::JsonStorageProviderConfig}};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    pub storage: StorageConfig,
    pub interface: InterfaceConfig,
}

pub trait ConfigProvider {
    type Error: std::error::Error;
    fn get(&self) -> impl std::future::Future<Output = Result<Config, Self::Error>> + Send;
}

impl Default for Config {
    fn default() -> Self {
        Self {
            storage: StorageConfig {
                provider: StorageConfigProvider::Json(
                    JsonStorageProviderConfig {
                        path: std::path::PathBuf::from("metrics.json"),
                        flush_on_write: true,
                    },
                ),
            },
            interface: InterfaceConfig {
                providers: crate::interface::InterfaceProvidersConfig {
                    http: Some(HttpInterfaceProviderConfig {
                        listeners: vec![
                            Listener::Tcp {
                                addr: "127.0.0.1".to_string(),
                                port: 25089,
                                fails: true,
                            },
                        ],
                    }),
                },
            },
        }
    }
}

pub async fn setup_config_provider(path: PathBuf) -> Result<impl ConfigProvider, Box<dyn std::error::Error>> {
    builtin::json::JsonConfigProvider::init(path)
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}
