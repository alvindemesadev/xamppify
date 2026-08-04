use crate::deployment::templates;
use crate::deployment::{BackupInfo, Deployment, DeploymentUpdate, GitInfo, RunOutput, VhostInfo};
use tauri::Manager;

fn app_data_dir(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    app.path().app_data_dir().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_deployments(app: tauri::AppHandle) -> Result<Vec<Deployment>, crate::error::AppError> {
    crate::deployment::list_deployments(&app_data_dir(&app)?).await
}

#[tauri::command]
pub async fn create_deployment(
    name: String,
    framework: String,
    app: tauri::AppHandle,
) -> Result<Deployment, crate::error::AppError> {
    crate::deployment::create_deployment(name, framework, &app_data_dir(&app)?).await
}

#[tauri::command]
pub async fn import_deployment(
    name: String,
    source_path: String,
    framework: Option<String>,
    app: tauri::AppHandle,
) -> Result<Deployment, crate::error::AppError> {
    crate::deployment::import_deployment(name, source_path, framework, &app_data_dir(&app)?).await
}

#[tauri::command]
pub async fn delete_deployment(name: String, app: tauri::AppHandle) -> Result<(), crate::error::AppError> {
    crate::deployment::delete_deployment(name, &app_data_dir(&app)?).await
}

#[tauri::command]
pub async fn backup_deployment(name: String, app: tauri::AppHandle) -> Result<String, crate::error::AppError> {
    let app_data_dir = app_data_dir(&app)?;
    let path = crate::deployment::backup_deployment(name, app_data_dir).await?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn update_deployment_meta(
    name: String,
    updates: DeploymentUpdate,
    app: tauri::AppHandle,
) -> Result<Deployment, crate::error::AppError> {
    crate::deployment::update_meta(name, updates, &app_data_dir(&app)?).await
}

#[tauri::command]
pub async fn open_deployment(name: String, app: tauri::AppHandle) -> Result<Deployment, crate::error::AppError> {
    crate::deployment::open_deployment(name, &app_data_dir(&app)?).await
}

#[tauri::command]
pub async fn duplicate_deployment(
    name: String,
    new_name: String,
    app: tauri::AppHandle,
) -> Result<Deployment, crate::error::AppError> {
    crate::deployment::duplicate_deployment(name, new_name, &app_data_dir(&app)?).await
}

#[tauri::command]
pub async fn git_import_deployment(
    name: String,
    repo_url: String,
    framework: Option<String>,
    app: tauri::AppHandle,
) -> Result<Deployment, crate::error::AppError> {
    crate::deployment::git_import(name, repo_url, framework, &app_data_dir(&app)?).await
}

#[tauri::command]
pub async fn set_custom_domain(
    name: String,
    domain: Option<String>,
    app: tauri::AppHandle,
) -> Result<Deployment, crate::error::AppError> {
    crate::deployment::set_custom_domain(name, domain, &app_data_dir(&app)?).await
}

#[tauri::command]
pub async fn toggle_vhost(
    name: String,
    enabled: bool,
    app: tauri::AppHandle,
) -> Result<Deployment, crate::error::AppError> {
    crate::deployment::toggle_vhost(name, enabled, &app_data_dir(&app)?).await
}

#[tauri::command]
pub async fn enable_deployment_ssl(
    name: String,
    app: tauri::AppHandle,
) -> Result<Deployment, crate::error::AppError> {
    crate::deployment::enable_ssl(name, &app_data_dir(&app)?).await
}

#[tauri::command]
pub async fn read_deployment_env(name: String) -> Result<String, crate::error::AppError> {
    crate::deployment::read_env(name).await
}

#[tauri::command]
pub async fn write_deployment_env(name: String, content: String) -> Result<(), crate::error::AppError> {
    crate::deployment::write_env(name, content).await
}

#[tauri::command]
pub async fn run_dependency_command(
    name: String,
    tool: String,
    action: String,
) -> Result<RunOutput, crate::error::AppError> {
    crate::deployment::run_dependency_command(name, tool, action).await
}

#[tauri::command]
pub async fn get_git_info(name: String) -> Result<GitInfo, crate::error::AppError> {
    crate::deployment::git_info(name).await
}

#[tauri::command]
pub async fn list_backups(
    name: Option<String>,
    app: tauri::AppHandle,
) -> Result<Vec<BackupInfo>, crate::error::AppError> {
    crate::deployment::list_backups(name, &app_data_dir(&app)?).await
}

#[tauri::command]
pub async fn restore_deployment_backup(
    name: String,
    backup_path: String,
    app: tauri::AppHandle,
) -> Result<Deployment, crate::error::AppError> {
    crate::deployment::restore_backup(name, backup_path, &app_data_dir(&app)?).await
}

#[tauri::command]
pub async fn get_frameworks() -> Vec<FrameworkInfo> {
    templates::FRAMEWORKS
        .iter()
        .map(|f| FrameworkInfo {
            id: f.to_string(),
            label: templates::framework_label(f).to_string(),
        })
        .collect()
}

#[tauri::command]
pub fn list_vhosts() -> Vec<VhostInfo> {
    crate::deployment::vhosts::list_vhosts()
        .into_iter()
        .map(|v| VhostInfo {
            domain: v.domain,
            root: v.root,
            port: v.port,
            ssl: v.ssl,
        })
        .collect()
}

#[tauri::command]
pub async fn get_detected_port() -> Result<PortInfo, String> {
    Ok(PortInfo {
        apache: crate::paths::apache_port(),
        mysql: crate::paths::mysql_port(),
    })
}

#[derive(Debug, serde::Serialize)]
pub struct FrameworkInfo {
    pub id: String,
    pub label: String,
}

#[derive(Debug, serde::Serialize)]
pub struct PortInfo {
    pub apache: u16,
    pub mysql: u16,
}

// Database provisioning helper
#[tauri::command]
pub async fn provision_database(
    deployment_name: String,
    database_name: Option<String>,
) -> Result<String, crate::error::AppError> {
    let db_name = database_name
        .unwrap_or_else(|| format!("{}_db", deployment_name.replace('-', "_")));
    crate::database::create_database(&db_name).await?;
    Ok(db_name)
}

// Meta store access for linking a DB after provisioning
#[tauri::command]
pub async fn set_linked_database(
    name: String,
    database_name: String,
    app: tauri::AppHandle,
) -> Result<Deployment, crate::error::AppError> {
    crate::deployment::update_meta(
        name,
        DeploymentUpdate {
            framework: None,
            tags: None,
            pinned: None,
            linked_db: Some(database_name),
            last_opened: false,
        },
        &app_data_dir(&app)?,
    )
    .await
}
