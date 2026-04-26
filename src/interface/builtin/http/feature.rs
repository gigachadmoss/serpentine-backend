use std::sync::Arc;
use tokio::sync::Mutex;

/// Holds and registers supported feature IDs for the HTTP interface.
#[derive(Clone)]
pub struct FeatureSupport {
    features: Arc<Mutex<Vec<String>>>,
}

impl FeatureSupport {
    pub fn new() -> Self {
        Self {
            features: Arc::new(Mutex::new(Vec::new())),
        }
    }
    /// Returns whether the given feature ID is supported.
    pub async fn supports(&self, id: &str) -> bool {
        let features = self.features.lock().await;

        features.iter().any(|f| f == id)
    }
    /// Returns the list of supported feature IDs.
    pub async fn get(&self) -> Vec<String> {
        let features = self.features.lock().await;

        features.clone()
    }
    /// Registers a new feature ID.
    pub async fn register(&self, id: String) {
        let mut features = self.features.lock().await;

        tracing::debug!("Registering feature ID: {}", id);

        features.push(id);
    }
}
