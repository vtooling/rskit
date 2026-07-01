use fast_log::{
    Config, Logger,
    consts::LogSize,
    error::LogError,
    plugin::file_split::{KeepType, Rolling, RollingType},
};
use log::LevelFilter;

/// Re-export the `log` crate so callers can use `rskit::log::info!` etc.
pub use log;

#[cfg(debug_assertions)]
pub const RUST_LOG: LevelFilter = LevelFilter::Debug;
#[cfg(not(debug_assertions))]
pub const RUST_LOG: LevelFilter = LevelFilter::Info;

/// Builder for the bundled `fast_log` logger.
pub struct Log {
    pub chan: Option<usize>,
    pub path: String,
    pub roll: Rolling,
    pub keep: KeepType,
    pub level: LevelFilter,
}

impl Log {
    /// Create a new logger configured to write to `<cwd>/log/app.log`.
    pub fn new() -> Self {
        let dir = std::env::current_dir()
            .ok()
            .and_then(|p| p.to_str().map(str::to_string))
            .unwrap_or_else(|| "./".to_string());
        Log {
            chan: Some(100_000),
            path: format!("{}/log/app.log", dir),
            roll: Rolling::new(RollingType::BySize(LogSize::MB(100))),
            keep: KeepType::KeepNum(10),
            level: RUST_LOG,
        }
    }

    /// Use a custom log file path.
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = path.into();
        self
    }

    /// Override the log level.
    pub fn with_level(mut self, level: LevelFilter) -> Self {
        self.level = level;
        self
    }

    /// Initialize a console-only logger.
    pub fn init(&self) -> Result<&'static Logger, LogError> {
        fast_log::init(
            Config::new()
                .level(self.level)
                .chan_len(self.chan)
                .console(),
        )
    }

    /// Initialize a logger writing to a single file (plus console).
    pub fn init_file(&self) -> Result<&'static Logger, LogError> {
        fast_log::init(
            Config::new()
                .level(self.level)
                .chan_len(self.chan)
                .file(&self.path)
                .console(),
        )
    }

    /// Initialize a rolling/split file logger (plus console).
    pub fn init_split(self) -> Result<&'static Logger, LogError> {
        fast_log::init(
            Config::new()
                .level(self.level)
                .chan_len(self.chan)
                .file_split(
                    &self.path,
                    self.roll,
                    self.keep,
                    fast_log::plugin::packer::LogPacker {},
                )
                .console(),
        )
    }
}

impl Default for Log {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_console_logs() {
        Log::new().init().unwrap();
        log::info!("rskit log init test");
        // give the async logger a moment to flush
        std::thread::sleep(std::time::Duration::from_millis(200));
    }

    #[test]
    fn builder_overrides() {
        let l = Log::new()
            .with_path("/tmp/rskit-test.log")
            .with_level(LevelFilter::Trace);
        assert_eq!(l.path, "/tmp/rskit-test.log");
        assert_eq!(l.level, LevelFilter::Trace);
    }
}
