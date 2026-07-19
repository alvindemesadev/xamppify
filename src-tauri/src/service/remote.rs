use crate::{ServiceState, ServiceStatus, ServiceType};
use std::time::Duration;
use tokio::net::TcpStream;

pub fn service_port(service_type: &ServiceType) -> u16 {
    match service_type {
        ServiceType::Apache => 80,
        ServiceType::MySQL => 3306,
        ServiceType::FileZilla => 21,
    }
}

pub async fn check_remote_service(
    ip: &str,
    service_type: &ServiceType,
) -> Result<ServiceStatus, String> {
    let port = service_port(service_type);
    let addr = format!("{}:{}", ip, port);

    let is_running = tokio::time::timeout(Duration::from_secs(3), TcpStream::connect(&addr))
        .await
        .is_ok_and(|r| r.is_ok());

    let status = if is_running {
        ServiceState::Running
    } else {
        ServiceState::Stopped
    };

    Ok(ServiceStatus {
        name: service_type.clone(),
        status,
        port,
        pid: None,
        uptime: None,
    })
}

pub async fn check_all_remote_services(ip: &str) -> Vec<ServiceStatus> {
    let services = vec![
        ServiceType::Apache,
        ServiceType::MySQL,
        ServiceType::FileZilla,
    ];
    let mut results = Vec::new();

    for svc in services {
        if let Ok(status) = check_remote_service(ip, &svc).await {
            results.push(status);
        }
    }

    results
}
