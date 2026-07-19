use crate::AppState;
use crate::{ServiceStatus, ServiceType};
use tauri::State;
use tracing::info;

fn parse_service_type(s: &str) -> Result<ServiceType, String> {
    match s.to_lowercase().as_str() {
        "apache" => Ok(ServiceType::Apache),
        "mysql" => Ok(ServiceType::MySQL),
        "filezilla" => Ok(ServiceType::FileZilla),
        _ => Err(format!("Unknown service: {}", s)),
    }
}

#[tauri::command]
pub async fn start_service(
    machine_id: String,
    service: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    info!("start_service: {} on {}", service, machine_id);

    let svc = parse_service_type(&service)?;

    let machine = state
        .registry
        .get(&machine_id)
        .await
        .ok_or_else(|| "Machine not found".to_string())?;

    let remote_host = (!is_local_machine(&machine.ip)).then_some(machine.ip.as_str());
    crate::service::control_service_on(&svc, crate::service::ServiceAction::Start, remote_host)
        .await
}

#[tauri::command]
pub async fn stop_service(
    machine_id: String,
    service: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    info!("stop_service: {} on {}", service, machine_id);

    let svc = parse_service_type(&service)?;

    let machine = state
        .registry
        .get(&machine_id)
        .await
        .ok_or_else(|| "Machine not found".to_string())?;

    let remote_host = (!is_local_machine(&machine.ip)).then_some(machine.ip.as_str());
    crate::service::control_service_on(&svc, crate::service::ServiceAction::Stop, remote_host).await
}

#[tauri::command]
pub async fn restart_service(
    machine_id: String,
    service: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    info!("restart_service: {} on {}", service, machine_id);

    let svc = parse_service_type(&service)?;

    let machine = state
        .registry
        .get(&machine_id)
        .await
        .ok_or_else(|| "Machine not found".to_string())?;

    let remote_host = (!is_local_machine(&machine.ip)).then_some(machine.ip.as_str());
    crate::service::control_service_on(&svc, crate::service::ServiceAction::Stop, remote_host)
        .await?;
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    crate::service::control_service_on(&svc, crate::service::ServiceAction::Start, remote_host)
        .await
}

#[tauri::command]
pub async fn get_service_status(
    machine_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<ServiceStatus>, String> {
    info!("get_service_status for {}", machine_id);

    let machine = state
        .registry
        .get(&machine_id)
        .await
        .ok_or_else(|| "Machine not found".to_string())?;

    let is_local = is_local_machine(&machine.ip);

    if is_local {
        let apache = crate::service::get_service_status(&ServiceType::Apache).await?;
        let mysql = crate::service::get_service_status(&ServiceType::MySQL).await?;
        let filezilla = crate::service::get_service_status(&ServiceType::FileZilla).await?;
        Ok(vec![apache, mysql, filezilla])
    } else {
        let services = crate::service::remote::check_all_remote_services(&machine.ip).await;
        Ok(services)
    }
}

fn is_local_machine(ip: &str) -> bool {
    if matches!(ip, "127.0.0.1" | "localhost" | "::1") {
        return true;
    }

    local_ip_address::local_ip()
        .map(|local_ip| local_ip.to_string() == ip)
        .unwrap_or(false)
}
