use crate::backup::BackupInfo;

#[tauri::command]
pub async fn create_backup(name: Option<String>) -> Result<BackupInfo, String> {
    crate::backup::create_backup(name).await
}

#[tauri::command]
pub async fn list_backups() -> Result<Vec<BackupInfo>, String> {
    crate::backup::list_backups().await
}

#[tauri::command]
pub async fn delete_backup(name: String) -> Result<(), String> {
    crate::backup::delete_backup(&name).await
}

#[tauri::command]
pub async fn dump_mysql() -> Result<String, String> {
    crate::backup::dump_mysql().await
}
