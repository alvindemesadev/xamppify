use crate::deployment::Deployment;
use tauri::Manager;

#[tauri::command]
pub async fn list_deployments() -> Result<Vec<Deployment>, crate::error::AppError> {
    crate::deployment::list_deployments().await
}

#[tauri::command]
pub async fn create_deployment(name: String, template: String) -> Result<Deployment, crate::error::AppError> {
    crate::deployment::create_deployment(name, template).await
}

#[tauri::command]
pub async fn import_deployment(name: String, source_path: String) -> Result<Deployment, crate::error::AppError> {
    crate::deployment::import_deployment(name, source_path).await
}

#[tauri::command]
pub async fn delete_deployment(name: String) -> Result<(), crate::error::AppError> {
    crate::deployment::delete_deployment(name).await
}

#[tauri::command]
pub async fn backup_deployment(name: String, app: tauri::AppHandle) -> Result<String, crate::error::AppError> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let path = crate::deployment::backup_deployment(name, app_data_dir).await?;
    Ok(path.to_string_lossy().to_string())
}
