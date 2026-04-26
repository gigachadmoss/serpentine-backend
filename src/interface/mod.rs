use serde::{Deserialize, Serialize};

pub mod builtin;

use builtin::{HttpInterfaceProvider, HttpInterfaceProviderConfig};

trait InterfaceProvider {
    type Error: std::error::Error;
    type Config: InterfaceProviderConfig;
    /// Initialize interface provider
    fn init(config: Self::Config) -> impl std::future::Future<Output = Result<Self, Self::Error>> + Send
    where
        Self: Sized;
    /// Shutdown interface provider
    fn shutdown(&mut self) -> impl std::future::Future<Output = ()> + Send;
    /// Waits for interface provider to die
    async fn wait(&mut self);
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct InterfaceProvidersConfig {
    pub http: Option<HttpInterfaceProviderConfig>,
}

trait InterfaceProviderConfig {}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct InterfaceConfig {
    pub providers: InterfaceProvidersConfig,
}

// Interface providers aren't accessible to other modules
pub async fn setup_providers(config: &InterfaceConfig) -> Result<tokio::sync::broadcast::Sender<()>, Box<dyn std::error::Error>> {
    let termination_tx = tokio::sync::broadcast::channel::<()>(1).0;

    if let Some(http_config) = &config.providers.http {
        let mut p = HttpInterfaceProvider::init(http_config.clone()).await?;

        let termination_inner_tx = termination_tx.clone();

        tokio::spawn(async move {
            p.wait().await;
            
            let _ = termination_inner_tx.send(());
        });
    }

    Ok(termination_tx)
}
