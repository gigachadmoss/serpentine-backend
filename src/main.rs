use serpentine_backend;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    tracing::info!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

    serpentine_backend::start_server().await;
}
