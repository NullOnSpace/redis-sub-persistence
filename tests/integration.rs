use redis_sub_persistence::config::AppConfig;
use serial_test::serial;
use std::io::Write;
use std::process::Command;
use std::time::Duration;

fn wait_for_line_in_file(path: &std::path::Path, contains: &str, timeout: Duration) -> bool {
    let start = std::time::Instant::now();
    while start.elapsed() < timeout {
        if path.exists()
            && let Ok(content) = std::fs::read_to_string(path)
        {
            for line in content.lines() {
                if line.contains(contains) {
                    return true;
                }
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    false
}

#[test]
fn config_load_failure_exits_with_error() {
    let result = AppConfig::load("/nonexistent/path.toml");
    assert!(result.is_err());
}

#[test]
#[serial]
fn end_to_end_subscribe_and_persist() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let data_dir = tmp_dir.path().join("data");
    let log_file = data_dir.join("messages.log");
    let config_file = tmp_dir.path().join("config.toml");

    let channel_name = "e2e-test-channel";

    let config_content = format!(
        r#"
[redis]
host = "127.0.0.1"
port = 6379
db = 0
password = ""
channel = ["{}"]

[persistence]
type = "file"
file = "{}"

[log]
level = "debug"
"#,
        channel_name,
        log_file.to_str().unwrap().replace('\\', "/")
    );

    let mut f = std::fs::File::create(&config_file).unwrap();
    f.write_all(config_content.as_bytes()).unwrap();

    let mut child = Command::new("./target/debug/redis-sub-persistence")
        .arg("--config")
        .arg(config_file.to_str().unwrap())
        .spawn()
        .expect("failed to start redis-sub-persistence, run `cargo build` first");

    std::thread::sleep(Duration::from_secs(3));

    let redis_publish_result = Command::new("redis-cli")
        .args(["publish", channel_name, "e2e-test-message"])
        .output()
        .expect("failed to run redis-cli, ensure redis is available");

    assert!(
        redis_publish_result.status.success(),
        "redis-cli publish failed: {}",
        String::from_utf8_lossy(&redis_publish_result.stderr)
    );

    let found = wait_for_line_in_file(&log_file, "e2e-test-message", Duration::from_secs(5));
    assert!(
        found,
        "message not found in persistence file within timeout"
    );

    let content = std::fs::read_to_string(&log_file).unwrap();
    assert!(content.contains("channel=e2e-test-channel"));
    assert!(content.contains("message=e2e-test-message"));

    child.kill().ok();
    child.wait().ok();
}

#[test]
#[serial]
fn end_to_end_multiple_channels() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let data_dir = tmp_dir.path().join("data");
    let log_file = data_dir.join("messages.log");
    let config_file = tmp_dir.path().join("config.toml");

    let config_content = format!(
        r#"
[redis]
host = "127.0.0.1"
port = 6379
db = 0
channel = ["e2e-multi-ch1", "e2e-multi-ch2"]

[persistence]
type = "file"
file = "{}"

[log]
level = "debug"
"#,
        log_file.to_str().unwrap().replace('\\', "/")
    );

    let mut f = std::fs::File::create(&config_file).unwrap();
    f.write_all(config_content.as_bytes()).unwrap();

    let mut child = Command::new("./target/debug/redis-sub-persistence")
        .arg("--config")
        .arg(config_file.to_str().unwrap())
        .spawn()
        .expect("failed to start redis-sub-persistence");

    std::thread::sleep(Duration::from_secs(3));

    Command::new("redis-cli")
        .args(["publish", "e2e-multi-ch1", "msg-from-ch1"])
        .output()
        .expect("redis-cli failed");

    Command::new("redis-cli")
        .args(["publish", "e2e-multi-ch2", "msg-from-ch2"])
        .output()
        .expect("redis-cli failed");

    let found1 = wait_for_line_in_file(&log_file, "msg-from-ch1", Duration::from_secs(5));
    let found2 = wait_for_line_in_file(&log_file, "msg-from-ch2", Duration::from_secs(5));
    assert!(found1, "msg-from-ch1 not found");
    assert!(found2, "msg-from-ch2 not found");

    let content = std::fs::read_to_string(&log_file).unwrap();
    assert!(content.contains("channel=e2e-multi-ch1"));
    assert!(content.contains("channel=e2e-multi-ch2"));

    child.kill().ok();
    child.wait().ok();
}
