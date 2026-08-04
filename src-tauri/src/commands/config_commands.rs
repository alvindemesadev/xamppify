use crate::config_editor::{ConfigFile, ConfigTestResult, IniSection};
use tauri::Manager;

#[tauri::command]
pub fn get_known_configs() -> Vec<ConfigFile> {
    crate::config_editor::known_configs()
}

#[tauri::command]
pub fn parse_ini_sections(content: String) -> Vec<IniSection> {
    crate::config_editor::parse_ini_sections(&content)
}

#[tauri::command]
pub async fn test_apache_config() -> Result<ConfigTestResult, String> {
    crate::config_editor::test_apache_config().await
}

#[tauri::command]
pub async fn save_config_file(
    path: String,
    content: String,
    app: tauri::AppHandle,
) -> Result<String, String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    crate::config_editor::save_config_file(std::path::Path::new(&path), &content, &app_data_dir).await
}
