use std::{collections::HashMap, path::PathBuf};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex;

use crate::metric::{Metric, MetricValue};

use super::super::StorageProvider;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to read file: {0}")]
    Read(std::io::Error),
    #[error("Failed to parse metrics from JSON: {0}")]
    Parse(serde_json::Error),
    #[error("Failed to write file: {0}")]
    Write(std::io::Error),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct JsonStorageProviderConfig {
    pub path: PathBuf,
    /// Whether to write the JSON file every write
    pub flush_on_write: bool,
}

pub struct JsonStorageProvider {
    path: PathBuf,
    metrics: Arc<Mutex<JsonMetricMap>>,
}

type JsonMetricMap = HashMap<String, Vec<JsonMetric>>;

/// (timestamp, value)
type JsonMetric = (i64, JsonMetricValue);

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
enum JsonMetricValue {
    Integer(i64),
    Float(f64),
    String(String),
}

impl From<MetricValue> for JsonMetricValue {
    fn from(value: MetricValue) -> Self {
        match value {
            MetricValue::Integer(i) => JsonMetricValue::Integer(i),
            MetricValue::Float(f) => JsonMetricValue::Float(f),
            MetricValue::String(s) => JsonMetricValue::String(s),
        }
    }
}

impl StorageProvider for JsonStorageProvider {
    type Error = Error;

    async fn write(&self, id: i64, value: MetricValue) -> Result<(), Self::Error> {
        {
            let mut metrics = self.metrics.lock().await;

            metrics.insert(id.to_string(), vec![(id, value.into())]);
        }

        self.write_metrics().await?;

        Ok(())
    }

    async fn read(&self, id: &str, timestamp: i64) -> Result<Option<Metric>, Self::Error> {
        Ok(None)
    }
}

impl JsonStorageProvider {
    pub async fn init(config: JsonStorageProviderConfig) -> Self {
        let mut provider = Self {
            path: config.path,
            metrics: Arc::new(Mutex::new(HashMap::new())),
        };
        provider.load_metrics().await.unwrap();
        provider
    }

    async fn setup(&self) -> Result<(), Error> {
        // Create JSON storage file if it doesn't exist
        if !self.path.is_file() {
            tracing::info!("JSON storage file does not exist, creating: {:?}", self.path);
            tokio::fs::File::create(&self.path).await.map_err(Error::Write)?;
        } else {
            tracing::info!("Loading existing JSON storage file: {:?}", self.path);
        }
        Ok(())
    }

    async fn load_metrics(&mut self) -> Result<(), Error> {
        tracing::info!("Reading metrics from JSON storage file: {:?}", self.path);
        
        let mut f = tokio::fs::File::open(&self.path).await.map_err(Error::Read)?;
        
        let mut metrics_raw = String::new();

        f.read_to_string(&mut metrics_raw).await.map_err(Error::Read)?;

        let metrics = serde_json::from_str::<JsonMetricMap>(&metrics_raw).map_err(Error::Parse)?;

        self.metrics.lock().await.extend(metrics);

        tracing::info!("Loaded metrics from JSON storage file: {:?}", self.path);

        Ok(())
    }

    async fn write_metrics(&self) -> Result<(), Error> {
        let metrics = self.metrics.lock().await;

        let metrics_raw = serde_json::to_string(&*metrics).map_err(Error::Parse)?;

        let mut f = tokio::fs::File::create(&self.path).await.map_err(Error::Write)?;

        f.write_all(metrics_raw.as_bytes()).await.map_err(Error::Write)?;

        Ok(())
    }
}