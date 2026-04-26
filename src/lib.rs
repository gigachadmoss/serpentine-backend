/// Connection to automotive data sources
mod auto;
/// Backend configuration
pub mod config;
/// Control thread
mod control;
/// External interaction
mod interface;
/// Metric handling and storage
mod metric;

use config::Config;

pub async fn start_server(config: Config) {
    tracing::info!("Starting serpentine server...");

    let i_termination_tx = match interface::setup_providers(&config.interface).await {
        Ok(tx) => tx,
        Err(e) => {
            tracing::error!("Failed to setup interface providers: {}", e);
            return;
        }
    };

    let storage_provider = metric::storage::setup_provider(config.storage).await;

    control::control_loop(i_termination_tx.subscribe()).await;
}
