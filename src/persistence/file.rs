use chrono::Local;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::error::AppError;

pub struct FilePersistence {
    #[allow(dead_code)]
    path: PathBuf,
    writer: Arc<Mutex<std::io::BufWriter<File>>>,
}

impl FilePersistence {
    pub fn new(path: &str) -> Result<Self, AppError> {
        let p = PathBuf::from(path);
        if let Some(parent) = p.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent).map_err(AppError::PersistDirCreate)?;
        }
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&p)
            .map_err(AppError::PersistFileOpen)?;
        Ok(Self {
            path: p,
            writer: Arc::new(Mutex::new(std::io::BufWriter::new(file))),
        })
    }

    pub async fn save(&self, channel: &str, message: &str) -> Result<(), AppError> {
        let channel = channel.to_string();
        let message = message.to_string();
        let writer = self.writer.clone();
        tokio::task::spawn_blocking(move || {
            let mut writer = writer.lock().map_err(AppError::file_lock_poisoned)?;
            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
            write!(writer, "[{}] channel={} message={}\n", timestamp, channel, message)
                .map_err(AppError::PersistWrite)?;
            writer.flush().map_err(AppError::PersistFlush)?;
            Ok(())
        })
        .await
        .map_err(AppError::BlockingTask)?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn new_creates_parent_directory() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let file_path = tmp_dir.path().join("subdir").join("messages.log");
        let persistence = FilePersistence::new(file_path.to_str().unwrap()).unwrap();
        assert!(tmp_dir.path().join("subdir").exists());
        assert_eq!(persistence.path, file_path);
    }

    #[test]
    fn new_without_parent_dir() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let file_path = tmp_dir.path().join("messages.log");
        let persistence = FilePersistence::new(file_path.to_str().unwrap()).unwrap();
        assert_eq!(persistence.path, file_path);
    }

    #[tokio::test]
    async fn save_writes_formatted_line() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let file_path = tmp_dir.path().join("messages.log");
        let persistence = FilePersistence::new(file_path.to_str().unwrap()).unwrap();

        persistence.save("test-channel", "hello world").await.unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("channel=test-channel"));
        assert!(content.contains("message=hello world"));
        assert!(content.starts_with("["));
        assert!(content.ends_with("\n"));
    }

    #[tokio::test]
    async fn save_appends_multiple_messages() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let file_path = tmp_dir.path().join("messages.log");
        let persistence = FilePersistence::new(file_path.to_str().unwrap()).unwrap();

        persistence.save("ch1", "msg1").await.unwrap();
        persistence.save("ch2", "msg2").await.unwrap();
        persistence.save("ch1", "msg3").await.unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[0].contains("channel=ch1") && lines[0].contains("message=msg1"));
        assert!(lines[1].contains("channel=ch2") && lines[1].contains("message=msg2"));
        assert!(lines[2].contains("channel=ch1") && lines[2].contains("message=msg3"));
    }

    #[tokio::test]
    async fn save_does_not_overwrite_existing_file() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let file_path = tmp_dir.path().join("messages.log");
        let persistence = FilePersistence::new(file_path.to_str().unwrap()).unwrap();

        persistence.save("ch1", "first").await.unwrap();
        persistence.save("ch2", "second").await.unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("first"));
        assert!(content.contains("second"));
    }

    #[tokio::test]
    async fn save_preserves_content_after_reopen() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let file_path = tmp_dir.path().join("messages.log");

        let persistence1 = FilePersistence::new(file_path.to_str().unwrap()).unwrap();
        persistence1.save("ch1", "from_p1").await.unwrap();

        let persistence2 = FilePersistence::new(file_path.to_str().unwrap()).unwrap();
        persistence2.save("ch2", "from_p2").await.unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("from_p1"));
        assert!(content.contains("from_p2"));
    }

    #[tokio::test]
    async fn timestamp_format_matches_pattern() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let file_path = tmp_dir.path().join("messages.log");
        let persistence = FilePersistence::new(file_path.to_str().unwrap()).unwrap();

        persistence.save("ch", "msg").await.unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        let line = content.lines().next().unwrap();
        let timestamp_part = line.split("channel=").next().unwrap();
        let re = regex::Regex::new(r"^\[\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}\] ").unwrap();
        assert!(re.is_match(timestamp_part));
    }
}