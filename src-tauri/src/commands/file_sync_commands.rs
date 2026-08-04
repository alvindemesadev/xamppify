use crate::file_sync::history::{self, SyncHistoryEntry};
use tauri::Manager;

#[tauri::command]
pub async fn sync_to_remote(
    app: tauri::AppHandle,
    source: String,
    destination: String,
    remote_host: String,
    username: Option<String>,
    password: Option<String>,
) -> Result<crate::file_sync::SyncResult, crate::error::AppError> {
    let result = crate::file_sync::sync_to_remote(
        &source,
        &destination,
        &remote_host,
        username.as_deref(),
        password.as_deref(),
    )
    .await?;

    let message = if result.success {
        format!("Synced to {}: {} items", remote_host, result.files_copied)
    } else {
        format!("Sync errors:\n{}", history::trim_message(&result.output))
    };
    let entry = SyncHistoryEntry {
        timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        result: if result.success { "success" } else { "error" }.to_string(),
        message,
        remote_host: remote_host.clone(),
        source: source.clone(),
        destination: destination.clone(),
    };
    if let Ok(app_data_dir) = app.path().app_data_dir() {
        let _ = history::append(app_data_dir, entry);
    }

    Ok(result)
}

#[tauri::command]
pub async fn test_remote_connection(
    remote_host: String,
    destination: String,
    username: Option<String>,
    password: Option<String>,
) -> Result<crate::file_sync::ConnectionTestResult, crate::error::AppError> {
    Ok(crate::file_sync::test_remote_connection(
        &remote_host,
        &destination,
        username.as_deref(),
        password.as_deref(),
    )
    .await)
}

#[tauri::command]
pub fn get_sync_history(app: tauri::AppHandle) -> Result<Vec<SyncHistoryEntry>, crate::error::AppError> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    history::load(app_data_dir).map_err(Into::into)
}

#[tauri::command]
pub fn clear_sync_history(app: tauri::AppHandle) -> Result<(), crate::error::AppError> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    history::clear(app_data_dir).map_err(Into::into)
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn set_scheduled_sync(
    app: tauri::AppHandle,
    state: tauri::State<'_, crate::AppState>,
    source: String,
    destination: String,
    remote_host: String,
    username: Option<String>,
    password: Option<String>,
    interval_minutes: u64,
) -> Result<(), String> {
    let config = crate::sync_scheduler::SyncScheduleConfig {
        source,
        destination,
        remote_host,
        username,
        password,
        interval_minutes,
    };
    state.scheduler.start(app, config)
}

#[tauri::command]
pub fn stop_scheduled_sync(
    state: tauri::State<'_, crate::AppState>,
) -> Result<(), String> {
    state.scheduler.stop()
}

#[tauri::command]
pub fn get_scheduled_sync(
    state: tauri::State<'_, crate::AppState>,
) -> Option<crate::sync_scheduler::SyncScheduleStatus> {
    state.scheduler.status()
}
