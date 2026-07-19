use crate::discovery::MachineRegistry;
use crate::{DiscoveryMethod, Machine};
use mdns_sd::{ServiceDaemon, ServiceEvent};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::Emitter;
use tracing::{error, info};

pub async fn start_mdns_discovery(
    registry: Arc<MachineRegistry>,
    app_handle: tauri::AppHandle,
    running: Arc<AtomicBool>,
) {
    info!("Starting mDNS discovery");

    let daemon = match ServiceDaemon::new() {
        Ok(d) => d,
        Err(e) => {
            error!("Failed to create mDNS daemon: {}", e);
            return;
        }
    };

    let receiver = match daemon.browse("_http._tcp.local.") {
        Ok(r) => r,
        Err(e) => {
            error!("Failed to start mDNS browser: {}", e);
            return;
        }
    };

    loop {
        if !running.load(Ordering::SeqCst) {
            info!("mDNS discovery cancelled");
            break;
        }

        let event = tokio::select! {
            event = receiver.recv_async() => event,
            _ = tokio::time::sleep(Duration::from_millis(500)) => continue,
        };

        match event {
            Ok(event) => match event {
                ServiceEvent::ServiceResolved(info) => {
                    let hostname = info.get_hostname();
                    let addresses = info.get_addresses();

                    let address = match addresses.iter().next() {
                        Some(a) => *a,
                        None => continue,
                    };

                    let port = info.get_port();

                    if port != 80 && port != 443 && port != 8080 {
                        continue;
                    }

                    info!("Discovered service: {} at {}:{}", hostname, address, port);

                    if is_apache_server(address, port).await {
                        let machine = Machine {
                            id: address.to_string(),
                            hostname: hostname.trim_end_matches('.').to_string(),
                            ip: address.to_string(),
                            os: None,
                            services: crate::service::remote::check_all_remote_services(
                                &address.to_string(),
                            )
                            .await,
                            last_seen: chrono::Utc::now().to_rfc3339(),
                            online: true,
                            discovered_via: DiscoveryMethod::MDns,
                        };

                        registry.add_or_update(machine.clone()).await;
                        let _ = app_handle.emit("machine-discovered", &machine);
                        let _ = app_handle.emit("machine-online", &machine);

                        info!("Emitted machine-discovered for {}", machine.hostname);
                    }
                }
                ServiceEvent::ServiceRemoved(_, full_name) => {
                    info!("Service removed: {}", full_name);
                }
                _ => {}
            },
            Err(_) => {
                info!("mDNS receiver closed");
                break;
            }
        }
    }

    let _ = daemon.shutdown();
}

async fn is_apache_server(ip: std::net::IpAddr, port: u16) -> bool {
    let url = format!("http://{}:{}", ip, port);
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
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
