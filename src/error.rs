#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("config file read failed: {0}")]
    ConfigFileRead(#[source] std::io::Error),

    #[error("config parse failed: {0}")]
    ConfigParse(#[source] toml::de::Error),

    #[error("log directory creation failed: {0}")]
    LogDirCreate(#[source] std::io::Error),

    #[error("log level filter invalid: {level}")]
    LogLevelInvalid { level: String },

    #[error("file path not configured")]
    FilePathMissing,

    #[error("persistence directory creation failed: {0}")]
    PersistDirCreate(#[source] std::io::Error),

    #[error("persistence file open failed: {0}")]
    PersistFileOpen(#[source] std::io::Error),

    #[error("persistence write failed: {0}")]
    PersistWrite(#[source] std::io::Error),

    #[error("persistence flush failed: {0}")]
    PersistFlush(#[source] std::io::Error),

    #[error("file lock poisoned: {0}")]
    FileLockPoisoned(String),

    #[error("redis connection failed: {0}")]
    RedisConnection(#[source] redis::RedisError),

    #[error("redis subscribe failed: {0}")]
    RedisSubscribe(#[source] redis::RedisError),

    #[error("redis payload decode failed: {0}")]
    RedisPayload(#[source] redis::RedisError),

    #[error("redis reconnect exceeded max attempts ({max})")]
    RedisMaxReconnects { max: u32 },

    #[error("blocking task failed: {0}")]
    BlockingTask(#[source] tokio::task::JoinError),
}

impl AppError {
    pub fn file_lock_poisoned<T>(e: std::sync::PoisonError<T>) -> Self {
        AppError::FileLockPoisoned(e.to_string())
    }
}
