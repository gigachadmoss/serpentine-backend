pub async fn control_loop(
    mut i_termination_rx: tokio::sync::broadcast::Receiver<()>
) {
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {}
        _ = i_termination_rx.recv() => {}
    }

    tracing::info!("Shutting down...");
}
