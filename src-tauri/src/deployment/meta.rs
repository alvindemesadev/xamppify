use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeploymentMeta {
    pub framework: Option<String>,
    pub tags: Vec<String>,
    pub pinned: bool,
    pub custom_domain: Option<String>,
    pub linked_db: Option<String>,
    pub vhost_enabled: bool,
    pub ssl_enabled: bool,
    pub last_opened: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MetaStore {
    pub deployments: HashMap<String, DeploymentMeta>,
}

impl MetaStore {
    pub fn load(path: &PathBuf) -> Self {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|content| serde_json::from_str(&content).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, path: &PathBuf) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create metadata folder: {e}"))?;
        }
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize deployment metadata: {e}"))?;
        std::fs::write(path, content).map_err(|e| format!("Failed to save deployment metadata: {e}"))
    }

    pub fn get(&self, name: &str) -> DeploymentMeta {
        self.deployments.get(name).cloned().unwrap_or_default()
    }

    pub fn set(&mut self, name: String, meta: DeploymentMeta) {
        self.deployments.insert(name, meta);
    }

    pub fn update<F: FnOnce(&mut DeploymentMeta)>(&mut self, name: &str, update: F) {
        let mut meta = self.get(name);
        update(&mut meta);
        self.deployments.insert(name.to_string(), meta);
    }

    pub fn remove(&mut self, name: &str) {
        self.deployments.remove(name);
    }
}

pub fn meta_file(app_data_dir: &std::path::Path) -> PathBuf {
    app_data_dir.join("deployment-meta.json")
}
