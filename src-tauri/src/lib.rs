mod commands;
mod config_editor;
mod credentials;
mod database;
mod deployment;
mod discovery;
mod error;
mod file_browser;
mod file_sync;
mod log;
mod paths;
mod performance;
mod recycle;
mod service;
mod ssl_manager;
mod sync_scheduler;

use discovery::MachineRegistry;
use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tauri::Manager;

fn init_tracing(app_data_dir: Option<std::path::PathBuf>) {
    use tracing_subscriber::prelude::*;

    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    let registry = tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer().with_target(false));

    let Some(log_dir) = app_data_dir.map(|dir| dir.join("logs")) else {
        registry.init();
        return;
    };
    if std::fs::create_dir_all(&log_dir).is_err() {
        registry.init();
        return;
    }
    let appender = tracing_appender::rolling::Builder::new()
        .rotation(tracing_appender::rolling::Rotation::DAILY)
        .filename_prefix("xamppify")
        .filename_suffix("log")
        .build(log_dir);
    let Ok(appender) = appender else {
        registry.init();
        return;
    };
    let (writer, guard) = tracing_appender::non_blocking(appender);
    Box::leak(Box::new(guard));
    registry.with(tracing_subscriber::fmt::layer().with_writer(writer)).init();
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiscoveryMethod {
    MDns,
    PortScan,
    Manual,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ServiceType {
    Apache,
    MySQL,
    FileZilla,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ServiceState {
    Running,
    Stopped,
    Starting,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStatus {
    pub name: ServiceType,
    pub status: ServiceState,
    pub port: u16,
    pub pid: Option<u32>,
    pub uptime: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Machine {
    pub id: String,
    pub hostname: String,
    pub ip: String,
    pub os: Option<String>,
    pub services: Vec<ServiceStatus>,
    pub last_seen: String,
    pub online: bool,
    pub discovered_via: DiscoveryMethod,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEvent {
    pub machine_id: String,
    pub service: ServiceType,
    pub status: ServiceState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogLine {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub modified: String,
    pub is_dir: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct AppHealth {
    pub xampp_root: String,
    pub xampp_available: bool,
    pub openssl_available: bool,
    pub apache_log_available: bool,
    pub mysql_log_available: bool,
}

pub struct AppState {
    pub registry: Arc<MachineRegistry>,
    pub heartbeat_tracker: Arc<discovery::heartbeat::HeartbeatTracker>,
    pub discovery_running: Arc<AtomicBool>,
    pub scheduler: Arc<sync_scheduler::Scheduler>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())

        .manage(AppState {
            registry: MachineRegistry::new_shared(),
            heartbeat_tracker: discovery::heartbeat::HeartbeatTracker::new_shared(),
            discovery_running: Arc::new(AtomicBool::new(false)),
            scheduler: Arc::new(sync_scheduler::Scheduler::new()),
        })
        .setup(|app| {
            init_tracing(app.path().app_data_dir().ok());

            #[cfg(desktop)]
            app.handle()
                .plugin(tauri_plugin_updater::Builder::new().build())?;

            use tauri::menu::{MenuBuilder, MenuItemBuilder};
            use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};

            let show_item = MenuItemBuilder::with_id("show", "Show/Hide").build(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "Quit").build(app)?;

            let menu = MenuBuilder::new(app)
                .item(&show_item)
                .separator()
                .item(&quit_item)
                .build()?;

            TrayIconBuilder::new()
                .menu(&menu)
                .on_menu_event(move |app_handle, event| match event.id().as_ref() {
                    "show" => {
                        if let Some(window) = app_handle.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        app_handle.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        if let Some(window) = tray.app_handle().get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::discovery_commands::start_discovery,
            commands::discovery_commands::stop_discovery,
            commands::discovery_commands::get_discovered_machines,
            commands::discovery_commands::get_xampp_root,
            commands::discovery_commands::get_app_health,
            commands::discovery_commands::add_manual_machine,
            commands::service_commands::start_service,
            commands::service_commands::stop_service,
            commands::service_commands::restart_service,
            commands::service_commands::get_service_status,
            commands::log_commands::get_logs,
            commands::log_commands::start_log_watcher,
            commands::file_commands::list_directory,
            commands::file_commands::read_file,
            commands::file_commands::write_file,
            commands::file_commands::create_folder,
            commands::file_commands::create_file,
            commands::file_commands::delete_path,
            commands::file_commands::rename_path,
            commands::file_commands::upload_files,
            commands::file_commands::upload_folder,
            commands::file_commands::upload_paths,
            commands::file_commands::read_image,
            commands::database_commands::mysql_connect,
            commands::database_commands::mysql_disconnect,
            commands::credentials_commands::save_mysql_credentials,
            commands::credentials_commands::get_mysql_credentials,
            commands::credentials_commands::delete_mysql_credentials,
            commands::config_commands::get_known_configs,
            commands::config_commands::parse_ini_sections,
            commands::config_commands::test_apache_config,
            commands::config_commands::save_config_file,
            commands::database_commands::list_databases,
            commands::database_commands::list_tables,
            commands::database_commands::run_query,
            commands::database_commands::export_database,
            commands::deployment_commands::list_deployments,
            commands::deployment_commands::create_deployment,
            commands::deployment_commands::import_deployment,
            commands::deployment_commands::delete_deployment,
            commands::deployment_commands::backup_deployment,
            commands::deployment_commands::update_deployment_meta,
            commands::deployment_commands::open_deployment,
            commands::deployment_commands::duplicate_deployment,
            commands::deployment_commands::git_import_deployment,
            commands::deployment_commands::set_custom_domain,
            commands::deployment_commands::toggle_vhost,
            commands::deployment_commands::enable_deployment_ssl,
            commands::deployment_commands::read_deployment_env,
            commands::deployment_commands::write_deployment_env,
            commands::deployment_commands::run_dependency_command,
            commands::deployment_commands::get_git_info,
            commands::deployment_commands::list_backups,
            commands::deployment_commands::restore_deployment_backup,
            commands::deployment_commands::get_frameworks,
            commands::deployment_commands::list_vhosts,
            commands::deployment_commands::get_detected_port,
            commands::deployment_commands::provision_database,
            commands::deployment_commands::set_linked_database,
            commands::ssl_commands::list_certificates,
            commands::ssl_commands::read_certificate,
            commands::ssl_commands::generate_self_signed,
            commands::file_sync_commands::sync_to_remote,
            commands::file_sync_commands::test_remote_connection,
            commands::file_sync_commands::get_sync_history,
            commands::file_sync_commands::clear_sync_history,
            commands::file_sync_commands::set_scheduled_sync,
            commands::file_sync_commands::stop_scheduled_sync,
            commands::file_sync_commands::get_scheduled_sync,
            commands::performance_commands::get_local_performance,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
