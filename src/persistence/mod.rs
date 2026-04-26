pub mod database;
pub mod file;

use crate::config::{PersistenceConfig, PersistenceType};
use crate::error::AppError;

pub enum Persistence {
    File(file::FilePersistence),
    Db(database::DatabasePersistence),
}

impl Persistence {
    pub async fn save(&self, channel: &str, message: &str) -> Result<(), AppError> {
        match self {
            Persistence::File(fp) => fp.save(channel, message).await,
            Persistence::Db(db) => db.save(channel, message).await,
        }
    }
}

pub fn create_persistence(config: &PersistenceConfig) -> Result<Persistence, AppError> {
    match &config.persistence_type {
        PersistenceType::File => {
            let path = config.file.as_deref().ok_or(AppError::FilePathMissing)?;
            Ok(Persistence::File(file::FilePersistence::new(path)?))
        }
        PersistenceType::Db => {
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
            Ok(Persistence::Db(database::DatabasePersistence::new(db_config)?))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{PersistenceConfig, PersistenceType};
    use crate::error::AppError;

    fn make_file_config(path: &str) -> PersistenceConfig {
        PersistenceConfig {
            persistence_type: PersistenceType::File,
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
            persistence_type: PersistenceType::Db,
            file: None,
            db_host: Some("127.0.0.1".to_string()),
            db_port: 3306,
            db_password: None,
            db_db: Some("test_db".to_string()),
            db_timeout: 30,
            db_retries: 3,
        }
    }

    #[tokio::test]
    async fn create_file_persistence() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let path = tmp_dir.path().join("messages.log");
        let config = make_file_config(path.to_str().unwrap());
        let persister = create_persistence(&config).unwrap();
        persister.save("test-ch", "test-msg").await.unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("channel=test-ch"));
        assert!(content.contains("message=test-msg"));
    }

    #[test]
    fn create_file_persistence_missing_path() {
        let config = PersistenceConfig {
            persistence_type: PersistenceType::File,
            file: None,
            db_host: None,
            db_port: 3306,
            db_password: None,
            db_db: None,
            db_timeout: 30,
            db_retries: 3,
        };
        let result = create_persistence(&config);
        assert!(matches!(result, Err(AppError::FilePathMissing)));
    }

    #[tokio::test]
    async fn create_db_persistence() {
        let config = make_db_config();
        let persister = create_persistence(&config).unwrap();
        persister.save("test-ch", "test-msg").await.unwrap();
    }
}