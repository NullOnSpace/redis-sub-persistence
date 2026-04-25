use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
pub struct AppConfig {
    pub redis: RedisConfig,
    pub persistence: PersistenceConfig,
    pub log: LogConfig,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct RedisConfig {
    pub host: String,
    #[serde(default = "default_redis_port")]
    pub port: u16,
    #[serde(default)]
    pub db: u64,
    #[serde(default)]
    pub password: String,
    pub channel: Vec<String>,
}

fn default_redis_port() -> u16 {
    6379
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct PersistenceConfig {
    #[serde(default = "default_persistence_type", rename = "type")]
    pub persistence_type: String,
    pub file: Option<String>,
    pub db_host: Option<String>,
    #[serde(default = "default_db_port")]
    pub db_port: u16,
    pub db_password: Option<String>,
    pub db_db: Option<String>,
    #[serde(default = "default_db_timeout")]
    pub db_timeout: u64,
    #[serde(default = "default_db_retries")]
    pub db_retries: u32,
}

fn default_persistence_type() -> String {
    "file".to_string()
}

fn default_db_port() -> u16 {
    3306
}

fn default_db_timeout() -> u64 {
    30
}

fn default_db_retries() -> u32 {
    3
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct LogConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    pub file: Option<String>,
}

fn default_log_level() -> String {
    "info".to_string()
}

impl AppConfig {
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        content.parse()
    }
}

impl std::str::FromStr for AppConfig {
    type Err = Box<dyn std::error::Error>;

    fn from_str(content: &str) -> Result<Self, Self::Err> {
        let config: AppConfig = toml::from_str(content)?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn parse_full_config() {
        let content = r#"
[redis]
host = "127.0.0.1"
port = 6380
db = 2
password = "secret"
channel = ["ch1", "ch2"]

[persistence]
type = "file"
file = "./data/test.log"

[log]
level = "debug"
file = "./logs/test.log"
"#;
        let config = AppConfig::from_str(content).unwrap();
        assert_eq!(
            config.redis,
            RedisConfig {
                host: "127.0.0.1".to_string(),
                port: 6380,
                db: 2,
                password: "secret".to_string(),
                channel: vec!["ch1".to_string(), "ch2".to_string()],
            }
        );
        assert_eq!(
            config.persistence,
            PersistenceConfig {
                persistence_type: "file".to_string(),
                file: Some("./data/test.log".to_string()),
                db_host: None,
                db_port: 3306,
                db_password: None,
                db_db: None,
                db_timeout: 30,
                db_retries: 3,
            }
        );
        assert_eq!(
            config.log,
            LogConfig {
                level: "debug".to_string(),
                file: Some("./logs/test.log".to_string()),
            }
        );
    }

    #[test]
    fn parse_minimal_config_defaults() {
        let content = r#"
[redis]
host = "localhost"
channel = ["test"]

[persistence]

[log]
"#;
        let config = AppConfig::from_str(content).unwrap();
        assert_eq!(config.redis.port, 6379);
        assert_eq!(config.redis.db, 0);
        assert_eq!(config.redis.password, "");
        assert_eq!(config.persistence.persistence_type, "file");
        assert_eq!(config.persistence.file, None);
        assert_eq!(config.persistence.db_port, 3306);
        assert_eq!(config.persistence.db_timeout, 30);
        assert_eq!(config.persistence.db_retries, 3);
        assert_eq!(config.log.level, "info");
        assert_eq!(config.log.file, None);
    }

    #[test]
    fn parse_missing_redis_host() {
        let content = r#"
[redis]
channel = ["test"]

[persistence]

[log]
"#;
        let result = AppConfig::from_str(content);
        assert!(result.is_err());
    }

    #[test]
    fn parse_missing_redis_channel() {
        let content = r#"
[redis]
host = "127.0.0.1"

[persistence]

[log]
"#;
        let result = AppConfig::from_str(content);
        assert!(result.is_err());
    }

    #[test]
    fn parse_invalid_toml() {
        let content = "this is not valid toml {{{}}}";
        let result = AppConfig::from_str(content);
        assert!(result.is_err());
    }

    #[test]
    fn parse_db_persistence_config() {
        let content = r#"
[redis]
host = "127.0.0.1"
channel = ["test"]

[persistence]
type = "db"
db_host = "db.example.com"
db_port = 5432
db_password = "dbpass"
db_db = "my_db"
db_timeout = 60
db_retries = 5

[log]
level = "warn"
"#;
        let config = AppConfig::from_str(content).unwrap();
        assert_eq!(config.persistence.persistence_type, "db");
        assert_eq!(
            config.persistence.db_host,
            Some("db.example.com".to_string())
        );
        assert_eq!(config.persistence.db_port, 5432);
        assert_eq!(config.persistence.db_password, Some("dbpass".to_string()));
        assert_eq!(config.persistence.db_db, Some("my_db".to_string()));
        assert_eq!(config.persistence.db_timeout, 60);
        assert_eq!(config.persistence.db_retries, 5);
    }

    #[test]
    fn load_from_file_not_found() {
        let result = AppConfig::load("/nonexistent/path/config.toml");
        assert!(result.is_err());
    }

    #[test]
    fn load_from_existing_file() {
        let content = r#"
[redis]
host = "127.0.0.1"
channel = ["test"]

[persistence]

[log]
"#;
        let tmp_dir = tempfile::tempdir().unwrap();
        let file_path = tmp_dir.path().join("test_config.toml");
        std::fs::write(&file_path, content).unwrap();
        let config = AppConfig::load(file_path.to_str().unwrap()).unwrap();
        assert_eq!(config.redis.host, "127.0.0.1");
    }
}
