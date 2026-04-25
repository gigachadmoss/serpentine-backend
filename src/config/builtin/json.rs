use std::path::PathBuf;

use tokio::io::AsyncReadExt;

use super::super::{Config, ConfigProvider};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Config file read error: {0}")]
    Read(std::io::Error),
    #[error("Config file write error: {0}")]
    Write(std::io::Error),
    #[error("Config parse error: {0}")]
    Parse(serde_json::Error),
    #[error("Config file not found at path: {0}")]
    NotFound(PathBuf),
}

pub struct JsonConfigProvider {
    path: String,
    config: Config,
}

impl ConfigProvider for JsonConfigProvider {
    type Error = Error;

    async fn get(&self) -> Result<Config, Self::Error> {
        Ok(self.config.clone())
    }
}

impl JsonConfigProvider {
    pub async fn init(path: PathBuf) -> Result<Self, Error> {
        tracing::debug!("Initializing JSON config provider with path: {:?}", path);

        if path.is_file() {
            tracing::info!("Found config file at {}", path.to_string_lossy().to_string());
            let mut f = tokio::fs::File::open(&path).await.map_err(Error::Read)?;
            let mut contents = String::new();

            // Read config file
            f.read_to_string(&mut contents).await.map_err(Error::Read)?;

            // Parse config file
            let config: Config = serde_json::from_str(&contents).map_err(Error::Parse)?;

            Ok(Self {
                path: path.to_string_lossy().into_owned(),
                config,
            })
        } else {
            Err(Error::NotFound(path))
        }
    }
}

/// Validates the config file at the given path
pub async fn validate_config(path: PathBuf) -> Result<(), Error> {
    tracing::debug!("Validating config file at path: {:?}", path);

    if path.is_file() {
        tracing::info!("Found config file at {}", path.to_string_lossy().to_string());
        let mut f = tokio::fs::File::open(&path).await.map_err(Error::Read)?;
        let mut contents = String::new();

        // Read config file
        f.read_to_string(&mut contents).await.map_err(Error::Read)?;

        // Parse config file
        serde_json::from_str::<Config>(&contents).map_err(Error::Parse)?;

        Ok(())
    } else {
        Err(Error::NotFound(path))
    }
}
