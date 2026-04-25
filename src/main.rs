use clap::Parser;
use redis_sub_persistence::config::AppConfig;
use redis_sub_persistence::logger;
use redis_sub_persistence::persistence;
use redis_sub_persistence::signal;
use redis_sub_persistence::subscriber;
use tracing::{error, info};

#[derive(Parser)]
#[command(name = "redis-sub-persistence")]
#[command(about = "Persist Redis Pub/Sub messages to file or database")]
struct Cli {
    #[arg(short, long, default_value = "config.toml")]
    config: String,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let app_config = match AppConfig::load(&cli.config) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("failed to load config from {}: {}", cli.config, e);
            std::process::exit(1);
        }
    };

    if let Err(e) = logger::init(&app_config.log) {
        eprintln!("failed to init logger: {}", e);
        std::process::exit(1);
    }

    info!("config loaded from {}", cli.config);
    info!(
        "redis: {}:{} channels={}",
        app_config.redis.host,
        app_config.redis.port,
        app_config.redis.channel.join(", ")
    );
    info!(
        "persistence: type={}",
        app_config.persistence.persistence_type
    );

    let persister = match persistence::create_persistence(&app_config.persistence) {
        Ok(p) => p,
        Err(e) => {
            error!("failed to create persistence: {}", e);
            std::process::exit(1);
        }
    };

    let (tx, rx) = tokio::sync::mpsc::channel::<()>(1);

    tokio::spawn(signal::watch_shutdown(tx));

    if let Err(e) = subscriber::run_subscriber(&app_config.redis, persister, rx).await {
        error!("subscriber error: {}", e);
        std::process::exit(1);
    }

    info!("redis-sub-persistence exited gracefully");
}
