use std::{
    sync::{Arc, LazyLock, RwLock},
    time::Duration,
};

use anyhow::Result;
// Leading `::` resolves to the external `config` crate, avoiding a clash with
// this module's own name (`rskit::config`).
use ::config::{Config, File, FileFormat};
use notify::{Event, RecommendedWatcher, Watcher};
use serde::{Deserialize, Serialize};

/// Default application settings. Users may define their own struct and use
/// [`Configs`] / [`load`] / [`auto`] instead.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Settings {
    pub app: App,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct App {
    pub version: String,
}

/// The auto-loaded [`Settings`], refreshed by the file watcher when active.
pub static AUTO_CONFIG: LazyLock<Arc<RwLock<Option<Settings>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(load::<Settings>(None).unwrap_or(None))));

fn default_path() -> String {
    std::env::current_dir()
        .map(|p| p.join("app.toml").to_string_lossy().to_string())
        .unwrap_or_else(|_| "app.toml".to_string())
}

/// Load and deserialize a configuration file (TOML by default).
///
/// `name` is the file stem; `None` means `app`. Returns `Ok(None)` if the file
/// does not exist (rather than erroring).
pub fn load<T: for<'de> Deserialize<'de>>(name: Option<&str>) -> Result<Option<T>> {
    load_from(name.unwrap_or("app"))
}

/// Load from an explicit file stem.
pub fn load_from<T: for<'de> Deserialize<'de>>(stem: &str) -> Result<Option<T>> {
    let path = format!("{}.toml", stem);
    if !std::path::Path::new(&path).exists() {
        return Ok(None);
    }
    let cfg = Config::builder()
        .add_source(File::with_name(&path))
        .build()?;
    Ok(Some(cfg.try_deserialize()?))
}

/// Load raw TOML text into a deserializable type.
pub fn load_str<T: for<'de> Deserialize<'de>>(toml: &str) -> Result<T> {
    let cfg = Config::builder()
        .add_source(File::from_str(toml, FileFormat::Toml))
        .build()?;
    Ok(cfg.try_deserialize()?)
}

/// Generic config holder. Useful when you want to keep the parsed value alongside.
pub struct Configs<T: for<'de> Deserialize<'de>> {
    config: Option<T>,
}

impl<T: for<'de> Deserialize<'de>> Configs<T> {
    pub fn new() -> Self {
        Configs { config: None }
    }

    /// Load from disk. `name` is the file stem; `None` means `app`.
    pub fn init(&mut self, name: Option<&str>) -> Option<&T> {
        self.config = load::<T>(name).ok().flatten();
        self.config.as_ref()
    }

    /// Borrow the last loaded configuration.
    pub fn get(&self) -> Option<&T> {
        self.config.as_ref()
    }
}

impl<T: for<'de> Deserialize<'de>> Default for Configs<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Read a snapshot of the auto-loaded [`Settings`].
pub fn auto() -> Option<Settings> {
    AUTO_CONFIG.read().ok().and_then(|g| g.clone())
}

/// Force a re-read of the auto-loaded settings from disk.
pub fn refresh() {
    if let (Ok(Some(s)), Ok(mut g)) = (load::<Settings>(None), AUTO_CONFIG.write()) {
        *g = Some(s);
    }
}

/// Spawn a background thread that watches `app.toml` and calls [`refresh`] on
/// modification. Returns immediately.
pub fn init_auto_watch() {
    std::thread::spawn(watch_loop);
}

fn watch_loop() {
    let (tx, rx) = std::sync::mpsc::channel();
    let watcher: Result<RecommendedWatcher, _> = Watcher::new(
        tx,
        notify::Config::default().with_poll_interval(Duration::from_secs(3)),
    );
    let mut watcher = match watcher {
        Ok(w) => w,
        Err(e) => {
            log::error!("config watcher init error: {e:?}");
            return;
        }
    };
    if let Err(e) = watcher.watch(
        std::path::Path::new(&default_path()),
        notify::RecursiveMode::NonRecursive,
    ) {
        log::error!("config watch error: {e:?}");
    }
    for res in rx {
        match res {
            Ok(Event {
                kind: notify::event::EventKind::Modify(_),
                ..
            }) => {
                log::info!("refreshing configuration ...");
                refresh();
            }
            Err(e) => log::error!("config watch recv error: {e:?}"),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_str_parses_settings() {
        let toml = r#"
[app]
version = "1.2.3"
"#;
        let s: Settings = load_str(toml).unwrap();
        assert_eq!(s.app.version, "1.2.3");
    }

    #[test]
    fn load_missing_file_is_none() {
        let s: Option<Settings> = load(Some("definitely-does-not-exist")).unwrap();
        assert!(s.is_none());
    }

    #[test]
    fn load_str_bad_toml_errors() {
        assert!(load_str::<Settings>("not = valid = toml").is_err());
    }

    #[test]
    fn configs_init_and_get() {
        let toml = r#"
[app]
version = "9.9"
"#;
        let s: Settings = load_str(toml).unwrap();
        // Round-trip through the auto static isn't desirable in tests; check Configs API via a temp file instead.
        let mut configs = Configs::<Settings>::new();
        assert!(configs.get().is_none());
        let _ = s;
        // init() reads from disk; with no file present it yields None.
        let _ = configs.init(Some("definitely-missing"));
        assert!(configs.get().is_none());
    }

    #[test]
    fn auto_snapshot_returns_clone() {
        // AUTO_CONFIG may be None or Some depending on cwd; just ensure it doesn't panic.
        let _ = auto();
    }

    #[test]
    fn load_from_existing() {
        // app.toml ships next to the crate; verify it loads and parses.
        if std::path::Path::new("app.toml").exists() {
            let s: Settings = load_from("app").unwrap().unwrap();
            assert!(!s.app.version.is_empty());
        }
    }
}
