pub mod database;
pub mod file;

use crate::config::PersistenceConfig;

pub trait Persistence: Send + Sync {
    fn save(&self, channel: &str, message: &str) -> Result<(), Box<dyn std::error::Error>>;
}

pub fn create_persistence(
    config: &PersistenceConfig,
) -> Result<Box<dyn Persistence>, Box<dyn std::error::Error>> {
    match config.persistence_type.as_str() {
        "file" => {
            let path = config.file.as_deref().ok_or("file path not configured")?;
            Ok(Box::new(file::FilePersistence::new(path)?))
        }
        "db" => {
            let db_config = database::DbConfig {
                host: config
                    .db_host
                    .clone()
                    .unwrap_or_else(|| "127.0.0.1".to_string()),
                port: config.db_port,
                password: config.db_password.clone(),
                db: config
                    .db_db
                    .clone()
                    .unwrap_or_else(|| "redis_messages".to_string()),
                timeout: config.db_timeout,
                retries: config.db_retries,
            };
            Ok(Box::new(database::DatabasePersistence::new(db_config)?))
        }
        other => Err(format!("unknown persistence type: {}", other).into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::PersistenceConfig;

    fn make_file_config(path: &str) -> PersistenceConfig {
        PersistenceConfig {
            persistence_type: "file".to_string(),
            file: Some(path.to_string()),
            db_host: None,
            db_port: 3306,
            db_password: None,
            db_db: None,
            db_timeout: 30,
            db_retries: 3,
        }
    }

    fn make_db_config() -> PersistenceConfig {
        PersistenceConfig {
            persistence_type: "db".to_string(),
            file: None,
            db_host: Some("127.0.0.1".to_string()),
            db_port: 3306,
            db_password: None,
            db_db: Some("test_db".to_string()),
            db_timeout: 30,
            db_retries: 3,
        }
    }

    #[test]
    fn create_file_persistence() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let path = tmp_dir.path().join("messages.log");
        let config = make_file_config(path.to_str().unwrap());
        let persister = create_persistence(&config).unwrap();
        persister.save("test-ch", "test-msg").unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("channel=test-ch"));
        assert!(content.contains("message=test-msg"));
    }

    #[test]
    fn create_file_persistence_missing_path() {
        let config = PersistenceConfig {
            persistence_type: "file".to_string(),
            file: None,
            db_host: None,
            db_port: 3306,
            db_password: None,
            db_db: None,
            db_timeout: 30,
            db_retries: 3,
        };
        let result = create_persistence(&config);
        assert!(result.is_err());
        let err_msg = match result {
            Err(e) => e.to_string(),
            Ok(_) => unreachable!(),
        };
        assert!(err_msg.contains("file path not configured"));
    }

    #[test]
    fn create_db_persistence() {
        let config = make_db_config();
        let persister = create_persistence(&config).unwrap();
        persister.save("test-ch", "test-msg").unwrap();
    }

    #[test]
    fn create_unknown_persistence_type() {
        let config = PersistenceConfig {
            persistence_type: "kafka".to_string(),
            file: None,
            db_host: None,
            db_port: 3306,
            db_password: None,
            db_db: None,
            db_timeout: 30,
            db_retries: 3,
        };
        let result = create_persistence(&config);
        assert!(result.is_err());
        let err_msg = match result {
            Err(e) => e.to_string(),
            Ok(_) => unreachable!(),
        };
        assert!(err_msg.contains("unknown persistence type: kafka"));
    }
}
