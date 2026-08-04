use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const HISTORY_FILE: &str = "sync_history.json";
const MAX_ENTRIES: usize = 50;
const MAX_MESSAGE_CHARS: usize = 1000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncHistoryEntry {
    pub timestamp: String,
    pub result: String,
    pub message: String,
    pub remote_host: String,
    pub source: String,
    pub destination: String,
}

fn history_path(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join(HISTORY_FILE)
}

pub fn load(app_data_dir: PathBuf) -> Result<Vec<SyncHistoryEntry>, String> {
    let path = history_path(&app_data_dir);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read sync history: {e}"))?;
    serde_json::from_str(&content).map_err(|e| format!("Failed to parse sync history: {e}"))
}

pub fn append(app_data_dir: PathBuf, entry: SyncHistoryEntry) -> Result<Vec<SyncHistoryEntry>, String> {
    let mut entries = load(app_data_dir.clone())?;
    entries.insert(0, entry);
    entries.truncate(MAX_ENTRIES);
    let path = history_path(&app_data_dir);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create history directory: {e}"))?;
    }
    let serialized = serde_json::to_string_pretty(&entries)
        .map_err(|e| format!("Failed to serialize sync history: {e}"))?;
    std::fs::write(&path, serialized).map_err(|e| format!("Failed to write sync history: {e}"))?;
    Ok(entries)
}

pub fn clear(app_data_dir: PathBuf) -> Result<(), String> {
    let path = history_path(&app_data_dir);
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| format!("Failed to clear sync history: {e}"))?;
    }
    Ok(())
}

/// Truncates long robocopy output so history stays readable in the UI.
pub fn trim_message(message: &str) -> String {
    let trimmed: String = message.chars().take(MAX_MESSAGE_CHARS).collect();
    if trimmed.len() < message.len() {
        format!("{trimmed}\n… (truncated)")
    } else {
        trimmed
    }
}
