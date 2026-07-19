use crate::AppState;
use crate::{AppHealth, DiscoveryMethod, Machine};
use std::sync::atomic::Ordering;
use std::time::Duration;
use tauri::Emitter;
use tauri::State;
use tokio::net::TcpStream;
use tracing::info;

#[tauri::command]
pub async fn start_discovery(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    info!("start_discovery called");

    if state.discovery_running.load(Ordering::SeqCst) {
        return Err("Discovery already running".to_string());
    }

    state.discovery_running.store(true, Ordering::SeqCst);

    let running = state.discovery_running.clone();
    let registry = state.registry.clone();
    let handle = app_handle.clone();

    tokio::spawn(async move {
        crate::discovery::mdns::start_mdns_discovery(registry, handle, running).await;
    });

    let running = state.discovery_running.clone();
    let registry = state.registry.clone();
    let handle = app_handle.clone();

    tokio::spawn(async move {
        crate::discovery::port_scanner::start_port_scan(registry, handle, running).await;
    });

    let running = state.discovery_running.clone();
    let registry = state.registry.clone();
    let heartbeat_tracker = state.heartbeat_tracker.clone();

    tokio::spawn(async move {
        crate::discovery::heartbeat::start_heartbeat(
            registry,
            app_handle,
            heartbeat_tracker,
            running,
        )
        .await;
    });

    Ok(())
}

#[tauri::command]
pub async fn stop_discovery(state: State<'_, AppState>) -> Result<(), String> {
    info!("stop_discovery called");

    if !state.discovery_running.load(Ordering::SeqCst) {
        return Err("Discovery is not running".to_string());
    }

    state.discovery_running.store(false, Ordering::SeqCst);
    info!("Discovery stopped");
    Ok(())
}

#[tauri::command]
pub async fn get_discovered_machines(state: State<'_, AppState>) -> Result<Vec<Machine>, String> {
    let machines = state.registry.get_all().await;
    Ok(machines)
}

#[tauri::command]
pub fn get_xampp_root() -> String {
    crate::paths::xampp_root().to_string_lossy().to_string()
}

#[tauri::command]
pub fn get_app_health() -> AppHealth {
    let root = crate::paths::xampp_root();
    AppHealth {
        xampp_root: root.to_string_lossy().to_string(),
        xampp_available: root.is_dir(),
        openssl_available: crate::paths::openssl_path().is_file(),
        apache_log_available: crate::paths::apache_log_path().is_file(),
        mysql_log_available: crate::paths::find_mysql_log_path().is_some(),
    }
}

#[tauri::command]
pub async fn add_manual_machine(
    ip: String,
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<Machine, String> {
    info!("Adding manual machine: {}", ip);

    let addr = format!("{}:80", ip);
    tokio::time::timeout(Duration::from_secs(3), TcpStream::connect(&addr))
        .await
        .map_err(|_| format!("Cannot connect to {}:80 - machine may be offline", ip))?
        .map_err(|e| format!("Connection error: {}", e))?;

    let machine = Machine {
        id: ip.clone(),
        hostname: ip.clone(),
        ip,
        os: None,
        services: Vec::new(),
        last_seen: chrono::Utc::now().to_rfc3339(),
        online: true,
        discovered_via: DiscoveryMethod::Manual,
    };

    state.registry.add_or_update(machine.clone()).await;
    let _ = app_handle.emit("machine-discovered", &machine);
    let _ = app_handle.emit("machine-online", &machine);

    Ok(machine)
}
