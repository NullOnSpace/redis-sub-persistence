use crate::config::RedisConfig;
use crate::persistence::Persistence;
use futures::StreamExt;
use redis::Client;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

pub async fn run_subscriber(
    config: &RedisConfig,
    persistence: Box<dyn Persistence>,
    mut shutdown_rx: mpsc::Receiver<()>,
) -> Result<(), Box<dyn std::error::Error>> {
    let url = if config.password.is_empty() {
        format!("redis://{}:{}/{}", config.host, config.port, config.db)
    } else {
        format!(
            "redis://:{}@{}:{}/{}",
            config.password, config.host, config.port, config.db
        )
    };

    info!("connecting to redis: {}", url);
    let client = Client::open(url.as_str())?;
    let mut pubsub = client.get_async_pubsub().await?;

    for channel in &config.channel {
        info!("subscribing to channel: {}", channel);
        pubsub.subscribe(channel).await?;
    }

    info!(
        "subscribed to {} channels, waiting for messages...",
        config.channel.len()
    );

    let mut msg_stream = pubsub.on_message();

    loop {
        tokio::select! {
            msg = msg_stream.next() => {
                match msg {
                    Some(msg) => {
                        let channel = msg.get_channel_name();
                        let payload: String = msg.get_payload()?;
                        info!("received message from channel {}: {}", channel, payload);
                        if let Err(e) = persistence.save(channel, &payload) {
                            error!("failed to persist message: {}", e);
                        }
                    }
                    None => {
                        warn!("pubsub connection closed, exiting subscriber");
                        break;
                    }
                }
            }
            _ = shutdown_rx.recv() => {
                info!("shutdown signal received, exiting subscriber");
                break;
            }
        }
    }

    Ok(())
}
