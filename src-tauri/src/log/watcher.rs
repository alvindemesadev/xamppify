use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;

pub struct LogWatcher {
    watcher: Mutex<Option<RecommendedWatcher>>,
}

impl LogWatcher {
    pub fn new() -> Self {
        Self {
            watcher: Mutex::new(None),
        }
    }

    pub async fn start(
        &self,
        app: AppHandle,
        source: &str,
        path: &Path,
    ) -> Result<(), String> {
        let source = source.to_string();
        let path = path.to_string_lossy().to_string();

        let watch_path = path.clone();
        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
                    let content = std::fs::read_to_string(&watch_path).unwrap_or_default();
                    let lines = if source == "Apache" {
                        crate::log::parser::parse_apache_log(&content, Some(100))
                    } else {
                        crate::log::parser::parse_mysql_log(&content, Some(100))
                    };
                    let _ = app.emit("log-update", serde_json::json!({
                        "source": &source,
                        "lines": lines,
                    }));
                }
            }
        })
        .map_err(|e| format!("Failed to create watcher: {e}"))?;

        watcher
            .watch(Path::new(&path), RecursiveMode::NonRecursive)
            .map_err(|e| format!("Failed to watch file: {e}"))?;

        let mut guard = self.watcher.lock().await;
        *guard = Some(watcher);
        Ok(())
    }
}
