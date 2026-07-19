use crate::LogLine;
use tracing::info;

#[tauri::command]
pub async fn get_logs(source: String, max_lines: Option<usize>) -> Result<Vec<LogLine>, String> {
    info!("get_logs: source={}", source);

    match source.to_lowercase().as_str() {
        "apache" => crate::log::tailer::read_apache_log(max_lines).await,
        "mysql" => crate::log::tailer::read_mysql_log(max_lines).await,
        _ => Err(format!("Unknown log source: {}", source)),
    }
}

#[tauri::command]
pub async fn start_log_watcher(
    app: tauri::AppHandle,
    source: String,
) -> Result<(), String> {
    info!("start_log_watcher: source={}", source);

    let path = match source.to_lowercase().as_str() {
        "apache" => std::path::PathBuf::from(r"C:\xampp\apache\logs\error.log"),
        "mysql" => std::path::PathBuf::from(r"C:\xampp\mysql\data\mysql.err"),
        _ => return Err(format!("Unknown log source: {}", source)),
    };

    let watcher = crate::log::watcher::LogWatcher::new();
    watcher.start(app, &source, &path).await
}
