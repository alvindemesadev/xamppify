pub mod heartbeat;
pub mod mdns;
pub mod port_scanner;

use crate::Machine;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct MachineRegistry {
    machines: RwLock<HashMap<String, Machine>>,
}

impl MachineRegistry {
    pub fn new() -> Self {
        Self {
            machines: RwLock::new(HashMap::new()),
        }
    }

    pub async fn add_or_update(&self, machine: Machine) {
        let mut machines = self.machines.write().await;
        match machines.get_mut(&machine.id) {
            Some(existing) => {
                let services = if machine.services.is_empty() {
                    existing.services.clone()
                } else {
                    machine.services
                };
                existing.hostname = machine.hostname;
                existing.ip = machine.ip;
                existing.os = machine.os;
                existing.services = services;
                existing.last_seen = machine.last_seen;
                existing.online = machine.online;
                existing.discovered_via = machine.discovered_via;
            }
            None => {
                machines.insert(machine.id.clone(), machine);
            }
        }
    }

    pub async fn remove(&self, id: &str) {
        let mut machines = self.machines.write().await;
        machines.remove(id);
    }

    pub async fn get_all(&self) -> Vec<Machine> {
        let machines = self.machines.read().await;
        machines.values().cloned().collect()
    }

    pub async fn get(&self, id: &str) -> Option<Machine> {
        let machines = self.machines.read().await;
        machines.get(id).cloned()
    }

    pub async fn mark_offline(&self, id: &str) {
        let mut machines = self.machines.write().await;
        if let Some(machine) = machines.get_mut(id) {
            machine.online = false;
        }
    }

    pub async fn mark_online(&self, id: &str) {
        let mut machines = self.machines.write().await;
        if let Some(machine) = machines.get_mut(id) {
            machine.online = true;
        }
    }

    pub fn new_shared() -> Arc<Self> {
        Arc::new(Self::new())
    }
}
