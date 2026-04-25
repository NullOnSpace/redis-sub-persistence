use super::Persistence;
use chrono::Local;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;

pub struct FilePersistence {
    path: String,
}

impl FilePersistence {
    pub fn new(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let p = Path::new(path);
        if let Some(parent) = p.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)?;
        }
        Ok(Self {
            path: path.to_string(),
        })
    }

    fn open_file(&self) -> Result<File, Box<dyn std::error::Error>> {
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|e| e.into())
    }
}

impl Persistence for FilePersistence {
    fn save(&self, channel: &str, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = self.open_file()?;
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        let line = format!("[{}] channel={} message={}\n", timestamp, channel, message);
        file.write_all(line.as_bytes())?;
        Ok(())
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
        assert_eq!(persistence.path, file_path.to_str().unwrap());
    }

    #[test]
    fn new_without_parent_dir() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let file_path = tmp_dir.path().join("messages.log");
        let persistence = FilePersistence::new(file_path.to_str().unwrap()).unwrap();
        assert_eq!(persistence.path, file_path.to_str().unwrap());
    }

    #[test]
    fn save_writes_formatted_line() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let file_path = tmp_dir.path().join("messages.log");
        let persistence = FilePersistence::new(file_path.to_str().unwrap()).unwrap();

        persistence.save("test-channel", "hello world").unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("channel=test-channel"));
        assert!(content.contains("message=hello world"));
        assert!(content.starts_with("["));
        assert!(content.ends_with("\n"));
    }

    #[test]
    fn save_appends_multiple_messages() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let file_path = tmp_dir.path().join("messages.log");
        let persistence = FilePersistence::new(file_path.to_str().unwrap()).unwrap();

        persistence.save("ch1", "msg1").unwrap();
        persistence.save("ch2", "msg2").unwrap();
        persistence.save("ch1", "msg3").unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[0].contains("channel=ch1") && lines[0].contains("message=msg1"));
        assert!(lines[1].contains("channel=ch2") && lines[1].contains("message=msg2"));
        assert!(lines[2].contains("channel=ch1") && lines[2].contains("message=msg3"));
    }

    #[test]
    fn save_does_not_overwrite_existing_file() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let file_path = tmp_dir.path().join("messages.log");
        let persistence = FilePersistence::new(file_path.to_str().unwrap()).unwrap();

        persistence.save("ch1", "first").unwrap();
        persistence.save("ch2", "second").unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("first"));
        assert!(content.contains("second"));
    }

    #[test]
    fn save_preserves_content_after_reopen() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let file_path = tmp_dir.path().join("messages.log");

        let persistence1 = FilePersistence::new(file_path.to_str().unwrap()).unwrap();
        persistence1.save("ch1", "from_p1").unwrap();

        let persistence2 = FilePersistence::new(file_path.to_str().unwrap()).unwrap();
        persistence2.save("ch2", "from_p2").unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("from_p1"));
        assert!(content.contains("from_p2"));
    }

    #[test]
    fn timestamp_format_matches_pattern() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let file_path = tmp_dir.path().join("messages.log");
        let persistence = FilePersistence::new(file_path.to_str().unwrap()).unwrap();

        persistence.save("ch", "msg").unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        let line = content.lines().next().unwrap();
        let timestamp_part = line.split("channel=").next().unwrap();
        let re = regex::Regex::new(r"^\[\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}\] ").unwrap();
        assert!(re.is_match(timestamp_part));
    }
}
