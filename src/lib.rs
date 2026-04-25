/// Backend configuration
pub mod config;
/// Connection to automotive data sources
mod auto;
/// Metric handling and storage
mod metric;
/// External interaction
mod interface;

use config::Config;

pub async fn start_server(config: Config) {
    tracing::info!("Starting serpentine server...");

    
}
