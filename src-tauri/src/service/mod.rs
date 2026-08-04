pub mod remote;

use crate::ServiceStatus;

pub enum ServiceAction {
    Start,
    Stop,
}

pub fn service_name(service_type: &crate::ServiceType) -> &'static str {
    match service_type {
        crate::ServiceType::Apache => "Apache2.4",
        crate::ServiceType::MySQL => "mysql",
        crate::ServiceType::FileZilla => "FileZilla",
    }
}

pub async fn control_service_on(
    service_type: &crate::ServiceType,
    action: ServiceAction,
    remote_host: Option<&str>,
) -> Result<(), String> {
    let name = service_name(service_type);

    let mut args = Vec::new();
    if let Some(host) = remote_host {
        args.push(format!(r"\\{host}"));
    }
    args.push(match action {
        ServiceAction::Start => "start".to_string(),
        ServiceAction::Stop => "stop".to_string(),
    });
    args.push(name.to_string());

    tokio::task::spawn_blocking(move || {
        let mut command = std::process::Command::new("sc");
        crate::paths::hide_console_for_std_command(&mut command);
        let output = command
            .args(args)
            .output()
            .map_err(|e| format!("Failed to run sc.exe: {}", e))?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let message = if stderr.trim().is_empty() {
                stdout.trim()
            } else {
                stderr.trim()
            };
            Err(format!("Service command failed: {}", message))
        }
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

pub async fn get_service_status(
    service_type: &crate::ServiceType,
) -> Result<ServiceStatus, String> {
    let name = service_name(service_type).to_string();
    let st = service_type.clone();

    tokio::task::spawn_blocking(move || {
        let mut command = std::process::Command::new("sc");
        crate::paths::hide_console_for_std_command(&mut command);
        let output = command
            .args(["query", &name])
            .output()
            .map_err(|e| format!("Failed to query service: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        parse_sc_query(&stdout, &st)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

fn parse_sc_query(
    output: &str,
    service_type: &crate::ServiceType,
) -> Result<ServiceStatus, String> {
    let state = if output.contains("RUNNING") {
        crate::ServiceState::Running
    } else if output.contains("STOPPED") {
        crate::ServiceState::Stopped
    } else if output.contains("START_PENDING") || output.contains("STOP_PENDING") {
        crate::ServiceState::Starting
    } else {
        crate::ServiceState::Error
    };

    let pid = output
        .lines()
        .find(|l| l.trim().starts_with("PID"))
        .and_then(|l| l.split(':').nth(1))
        .and_then(|s| s.trim().parse::<u32>().ok());

    let port = match service_type {
        crate::ServiceType::Apache => crate::paths::apache_port(),
        crate::ServiceType::MySQL => 3306,
        crate::ServiceType::FileZilla => 21,
    };

    Ok(ServiceStatus {
        name: service_type.clone(),
        status: state,
        port,
        pid,
        uptime: None,
    })
}
