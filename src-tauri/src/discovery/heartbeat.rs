use crate::discovery::MachineRegistry;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::Emitter;
use tokio::net::TcpStream;
use tokio::sync::RwLock;
use tracing::{info, warn};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(10);
const MAX_MISSED_PINGS: u32 = 3;

pub struct HeartbeatTracker {
    missed_pings: RwLock<HashMap<String, u32>>,
}

impl HeartbeatTracker {
    pub fn new() -> Self {
        Self {
            missed_pings: RwLock::new(HashMap::new()),
        }
    }

    pub fn new_shared() -> Arc<Self> {
        Arc::new(Self::new())
    }
}

pub async fn start_heartbeat(
    registry: Arc<MachineRegistry>,
    app_handle: tauri::AppHandle,
    tracker: Arc<HeartbeatTracker>,
    running: Arc<AtomicBool>,
) {
    info!("Starting heartbeat system");

    let mut interval = tokio::time::interval(HEARTBEAT_INTERVAL);

    while running.load(Ordering::SeqCst) {
        interval.tick().await;

        let machines = registry.get_all().await;

        for machine in machines {
            if !running.load(Ordering::SeqCst) {
                break;
            }

            let ip = machine.ip.clone();
            let id = machine.id.clone();
            let was_online = machine.online;

            let is_alive = tokio::time::timeout(
                Duration::from_secs(3),
                TcpStream::connect(format!("{}:{}", ip, 80)),
            )
            .await
            .is_ok_and(|result| result.is_ok());

            if is_alive {
                let mut missed = tracker.missed_pings.write().await;
                missed.remove(&id);

                if !was_online {
                    info!("Machine back online: {}", machine.hostname);
                    registry.mark_online(&id).await;
                    let machine = registry.get(&id).await;
                    if let Some(m) = machine {
                        let _ = app_handle.emit("machine-online", &m);
                    }
                }
            } else {
                let mut missed = tracker.missed_pings.write().await;
                let count = missed.entry(id.clone()).or_insert(0);
                *count += 1;

                if *count >= MAX_MISSED_PINGS && was_online {
                    warn!("Machine offline ({} missed): {}", count, machine.hostname);
                    registry.mark_offline(&id).await;
                    let machine = registry.get(&id).await;
                    if let Some(m) = machine {
                        let _ = app_handle.emit("machine-offline", &m);
                    }
                }
            }
        }
    }

    info!("Heartbeat system stopped");
}
