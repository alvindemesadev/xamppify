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
