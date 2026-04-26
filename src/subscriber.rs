use crate::config::RedisConfig;
use crate::error::AppError;
use crate::persistence::Persistence;
use futures::StreamExt;
use redis::Client;
use tokio::sync::mpsc;
use tokio::time::{Duration, sleep};
use tracing::{error, info, warn};

fn build_redis_url(config: &RedisConfig) -> String {
    if config.password.is_empty() {
        format!("redis://{}:{}/{}", config.host, config.port, config.db)
    } else {
        format!(
            "redis://:{}@{}:{}/{}",
            config.password, config.host, config.port, config.db
        )
    }
}

fn mask_redis_url(url: &str) -> String {
    if url.contains('@') {
        let parts: Vec<&str> = url.split('@').collect();
        let auth = parts[0];
        let rest = parts[1];
        let masked_auth = if auth.contains(':') {
            let auth_parts: Vec<&str> = auth.split(':').collect();
            format!("{}://:<masked>", auth_parts[0])
        } else {
            format!("{}://<masked>", auth)
        };
        format!("{}@{}", masked_auth, rest)
    } else {
        url.to_string()
    }
}

async fn connect_with_retry(config: &RedisConfig) -> Result<redis::aio::PubSub, AppError> {
    let url = build_redis_url(config);
    info!("connecting to redis: {}", mask_redis_url(&url));
    let client = Client::open(url.as_str()).map_err(AppError::RedisConnection)?;

    let max_retries = 10;
    let mut retry_count = 0;
    let base_delay = Duration::from_secs(1);

    loop {
        match client.get_async_pubsub().await {
            Ok(pubsub) => {
                info!("redis connection established");
                return Ok(pubsub);
            }
            Err(e) => {
                retry_count += 1;
                if retry_count >= max_retries {
                    return Err(AppError::RedisConnection(e));
                }
                let delay = base_delay * 2u32.pow(retry_count as u32 - 1);
                warn!(
                    "redis connection failed (attempt {}), retrying in {:?}: {}",
                    retry_count, delay, e
                );
                sleep(delay).await;
            }
        }
    }
}

pub async fn run_subscriber(
    config: &RedisConfig,
    persistence: Persistence,
    mut shutdown_rx: mpsc::Receiver<()>,
) -> Result<(), AppError> {
    let mut pubsub = connect_with_retry(config).await?;

    for channel in &config.channel {
        info!("subscribing to channel: {}", channel);
        pubsub
            .subscribe(channel)
            .await
            .map_err(AppError::RedisSubscribe)?;
    }

    info!(
        "subscribed to {} channels, waiting for messages...",
        config.channel.len()
    );

    let mut msg_stream = pubsub.on_message();
    let mut reconnect_count = 0;
    let max_reconnects = 100;

    loop {
        tokio::select! {
            msg = msg_stream.next() => {
                match msg {
                    Some(msg) => {
                        let channel = msg.get_channel_name();
                        let payload: String = msg.get_payload().map_err(AppError::RedisPayload)?;
                        info!("received message from channel {}: {}", channel, payload);
                        if let Err(e) = persistence.save(channel, &payload).await {
                            error!("failed to persist message: {}", e);
                        }
                    }
                    None => {
                        reconnect_count += 1;
                        if reconnect_count > max_reconnects {
                            error!("exceeded max reconnect attempts ({})", max_reconnects);
                            return Err(AppError::RedisMaxReconnects { max: max_reconnects });
                        }
                        warn!("pubsub connection closed, reconnecting (attempt {})...", reconnect_count);
                        let delay = Duration::from_secs(2u64.pow(reconnect_count.min(6)));
                        sleep(delay).await;
                        drop(msg_stream);
                        pubsub = connect_with_retry(config).await?;
                        for channel in &config.channel {
                            info!("subscribing to channel: {}", channel);
                            pubsub.subscribe(channel).await.map_err(AppError::RedisSubscribe)?;
                        }
                        msg_stream = pubsub.on_message();
                        info!("reconnected and resubscribed to {} channels", config.channel.len());
                        continue;
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
