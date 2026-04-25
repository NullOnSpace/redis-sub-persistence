use super::Persistence;
use tracing::{error, info};

#[allow(dead_code)]
pub struct DbConfig {
    pub host: String,
    pub port: u16,
    pub password: Option<String>,
    pub db: String,
    pub timeout: u64,
    pub retries: u32,
}

#[allow(dead_code)]
pub struct DatabasePersistence {
    config: DbConfig,
}

impl DatabasePersistence {
    pub fn new(config: DbConfig) -> Result<Self, Box<dyn std::error::Error>> {
        info!(
            "database persistence configured: {}:{}/{}",
            config.host, config.port, config.db
        );
        Ok(Self { config })
    }
}

impl Persistence for DatabasePersistence {
    fn save(&self, channel: &str, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        error!(
            "database persistence is not yet implemented, message from channel {} will be logged only",
            channel
        );
        println!("[DB-PERSIST] channel={} message={}", channel, message);
        Ok(())
    }
}
