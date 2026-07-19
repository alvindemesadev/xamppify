use crate::deployment::Deployment;

#[tauri::command]
pub async fn list_deployments() -> Result<Vec<Deployment>, String> {
    crate::deployment::list_deployments().await
}

#[tauri::command]
pub async fn create_deployment(name: String, template: String) -> Result<Deployment, String> {
    crate::deployment::create_deployment(name, template).await
}

#[tauri::command]
pub async fn import_deployment(name: String, source_path: String) -> Result<Deployment, String> {
    crate::deployment::import_deployment(name, source_path).await
}

#[tauri::command]
pub async fn delete_deployment(name: String) -> Result<(), String> {
    crate::deployment::delete_deployment(name).await
}
