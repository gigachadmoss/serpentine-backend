/// Backend configuration
mod config;
/// Connection to automotive data sources
mod auto;
/// Metric handling and storage
mod metric;
/// External interaction
mod interface;

pub async fn start_server() {
    tracing::info!("Starting serpentine server...");

    
}
