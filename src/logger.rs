use crate::config::LogConfig;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

pub fn init(config: &LogConfig) -> Result<(), Box<dyn std::error::Error>> {
    let filter = EnvFilter::try_new(&config.level).or_else(|_| EnvFilter::try_new("info"))?;

    if let Some(log_file) = &config.file {
        let path = std::path::Path::new(log_file);
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)?;
        }
        let dir = path.parent().unwrap_or(std::path::Path::new("."));
        let file_name = path.file_name().unwrap_or(std::ffi::OsStr::new("app.log"));
        let file_appender = tracing_appender::rolling::never(dir, file_name);

        tracing_subscriber::registry()
            .with(filter.clone())
            .with(fmt::layer().with_writer(std::io::stdout))
            .with(fmt::layer().with_ansi(false).with_writer(file_appender))
            .init();
    } else {
        tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer())
            .init();
    }

    Ok(())
}
