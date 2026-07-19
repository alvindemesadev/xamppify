use crate::discovery::MachineRegistry;
use crate::{DiscoveryMethod, Machine};
use std::net::{Ipv4Addr, SocketAddrV4};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::Emitter;
use tokio::net::TcpStream;
use tracing::info;

pub async fn start_port_scan(
    registry: Arc<MachineRegistry>,
    app_handle: tauri::AppHandle,
    running: Arc<AtomicBool>,
) {
    info!("Starting port scan discovery");

    let local_ip = match local_ip_address::local_ip() {
        Ok(ip) => ip,
        Err(e) => {
            tracing::error!("Failed to get local IP: {}", e);
            return;
        }
    };

    let subnet = match get_subnet(local_ip) {
        Some(subnet) => subnet,
        None => {
            tracing::warn!("No local IPv4 address is available; skipping IPv4 subnet scan");
            return;
        }
    };
    info!(
        "Scanning subnet: {}.{}.{}.0/24",
        subnet[0], subnet[1], subnet[2]
    );

    let ips: Vec<Ipv4Addr> = (1..255)
        .map(|last_octet| Ipv4Addr::new(subnet[0], subnet[1], subnet[2], last_octet))
        .collect();
    let total = ips.len();
    let scanned = Arc::new(AtomicUsize::new(0));
    let _ = app_handle.emit(
        "discovery-progress",
        serde_json::json!({ "scanned": 0, "total": total }),
    );

    let mut handles = Vec::new();

    for chunk in ips.chunks(32) {
        if !running.load(Ordering::SeqCst) {
            info!("Port scan cancelled");
            return;
        }

        let chunk = chunk.to_vec();
        let registry = registry.clone();
        let app_handle = app_handle.clone();
        let running = running.clone();
        let scanned = scanned.clone();

        let handle = tokio::spawn(async move {
            for ip in chunk {
                if !running.load(Ordering::SeqCst) {
                    break;
                }

                if check_port(ip, 80).await && is_apache_http(ip).await {
                    info!("Found XAMPP machine at {}", ip);

                    let machine = Machine {
                        id: ip.to_string(),
                        hostname: ip.to_string(),
                        ip: ip.to_string(),
                        os: None,
                        services: crate::service::remote::check_all_remote_services(
                            &ip.to_string(),
                        )
                        .await,
                        last_seen: chrono::Utc::now().to_rfc3339(),
                        online: true,
                        discovered_via: DiscoveryMethod::PortScan,
                    };

                    registry.add_or_update(machine.clone()).await;
                    let _ = app_handle.emit("machine-discovered", &machine);
                    let _ = app_handle.emit("machine-online", &machine);
                }

                let completed = scanned.fetch_add(1, Ordering::SeqCst) + 1;
                let _ = app_handle.emit(
                    "discovery-progress",
                    serde_json::json!({ "scanned": completed, "total": total }),
                );
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await;
    }

    info!("Port scan completed");
}

fn get_subnet(ip: std::net::IpAddr) -> Option<[u8; 3]> {
    match ip {
        std::net::IpAddr::V4(v4) => {
            let octets = v4.octets();
            Some([octets[0], octets[1], octets[2]])
        }
        std::net::IpAddr::V6(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::get_subnet;
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    #[test]
    fn derives_ipv4_subnet_without_parsing() {
        assert_eq!(
            get_subnet(IpAddr::V4(Ipv4Addr::new(192, 168, 4, 22))),
            Some([192, 168, 4])
        );
    }

    #[test]
    fn does_not_scan_loopback_when_only_ipv6_is_available() {
        assert_eq!(get_subnet(IpAddr::V6(Ipv6Addr::LOCALHOST)), None);
    }
}

async fn check_port(ip: Ipv4Addr, port: u16) -> bool {
    let addr = SocketAddrV4::new(ip, port);
    tokio::time::timeout(Duration::from_millis(500), TcpStream::connect(addr))
        .await
        .is_ok_and(|r| r.is_ok())
}

async fn is_apache_http(ip: Ipv4Addr) -> bool {
    let url = format!("http://{}/", ip);
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .danger_accept_invalid_certs(true)
        .build()
        .ok();

    let client = match client {
        Some(c) => c,
        None => return false,
    };

    match client.get(&url).send().await {
        Ok(response) => {
            let server_header = response
                .headers()
                .get(reqwest::header::SERVER)
                .and_then(|v| v.to_str().ok())
                .unwrap_or("")
                .to_lowercase();

            server_header.contains("apache") || server_header.contains("xampp")
        }
        Err(_) => false,
    }
}
