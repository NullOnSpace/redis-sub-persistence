use tokio::signal;
use tokio::sync::mpsc;
use tracing::info;

pub async fn watch_shutdown(tx: mpsc::Sender<()>) {
    match signal::ctrl_c().await {
        Ok(()) => info!("received SIGINT (ctrl-c), shutting down"),
        Err(e) => info!("error listening for shutdown signal: {}", e),
    }
    let _ = tx.send(()).await;
}
