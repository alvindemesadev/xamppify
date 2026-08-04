use crate::file_sync::history::{self, SyncHistoryEntry};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use tracing::{error, info};

#[derive(Clone, Debug)]
pub struct SyncScheduleConfig {
    pub source: String,
    pub destination: String,
    pub remote_host: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub interval_minutes: u64,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct SyncScheduleStatus {
    pub source: String,
    pub destination: String,
    pub remote_host: String,
    pub interval_minutes: u64,
    pub last_run: Option<String>,
    pub next_run: Option<String>,
}

struct ScheduleRuns {
    last_run: Option<String>,
    next_run: Option<String>,
}

struct ActiveSchedule {
    config: SyncScheduleConfig,
    handle: tokio::task::JoinHandle<()>,
}

pub struct Scheduler {
    active: Mutex<Option<ActiveSchedule>>,
    runs: Arc<Mutex<ScheduleRuns>>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            active: Mutex::new(None),
            runs: Arc::new(Mutex::new(ScheduleRuns {
                last_run: None,
                next_run: None,
            })),
        }
    }

    pub fn start(&self, app: AppHandle, config: SyncScheduleConfig) -> Result<(), String> {
        if config.interval_minutes == 0 {
            return Err("The sync interval must be at least 1 minute".to_string());
        }
        if config.source.trim().is_empty() || config.destination.trim().is_empty() || config.remote_host.trim().is_empty() {
            return Err("Source, destination, and remote host are required".to_string());
        }

        let mut active = self
            .active
            .lock()
            .map_err(|_| "Scheduler state is unavailable".to_string())?;
        if let Some(existing) = active.take() {
            existing.handle.abort();
        }

        let interval_minutes = config.interval_minutes;
        let next_run = chrono::Local::now()
            .checked_add_signed(chrono::Duration::minutes(interval_minutes as i64))
            .map(|time| time.format("%Y-%m-%d %H:%M:%S").to_string());
        if let Ok(mut runs) = self.runs.lock() {
            runs.last_run = None;
            runs.next_run = next_run;
        }

        let app_for_task = app.clone();
        let config_for_task = config.clone();
        let runs = Arc::clone(&self.runs);
        let handle = tokio::spawn(async move {
            loop {
                let started = chrono::Local::now();
                let result = crate::file_sync::sync_to_remote(
                    &config_for_task.source,
                    &config_for_task.destination,
                    &config_for_task.remote_host,
                    config_for_task.username.as_deref(),
                    config_for_task.password.as_deref(),
                )
                .await;

                let (success, files_copied, message) = match &result {
                    Ok(sync) if sync.success => (
                        true,
                        sync.files_copied,
                        format!(
                            "Scheduled sync to {}: {} items",
                            config_for_task.remote_host, sync.files_copied
                        ),
                    ),
                    Ok(sync) => (
                        false,
                        sync.files_copied,
                        format!(
                            "Scheduled sync errors:\n{}",
                            history::trim_message(&sync.output)
                        ),
                    ),
                    Err(err) => (false, 0, format!("Scheduled sync failed: {err}")),
                };
                let timestamp = started.format("%Y-%m-%d %H:%M:%S").to_string();

                let entry = SyncHistoryEntry {
                    timestamp: timestamp.clone(),
                    result: if success { "success" } else { "error" }.to_string(),
                    message: message.clone(),
                    remote_host: config_for_task.remote_host.clone(),
                    source: config_for_task.source.clone(),
                    destination: config_for_task.destination.clone(),
                };
                if let Ok(app_data_dir) = app_for_task.path().app_data_dir() {
                    let _ = history::append(app_data_dir, entry);
                }

                let _ = app_for_task.emit(
                    "scheduled-sync-result",
                    serde_json::json!({
                        "success": success,
                        "files_copied": files_copied,
                        "message": message,
                        "timestamp": timestamp,
                    }),
                );

                if let Ok(mut runs) = runs.lock() {
                    runs.last_run = Some(timestamp);
                }

                if success {
                    info!("scheduled sync to {} completed", config_for_task.remote_host);
                } else {
                    error!("scheduled sync to {} reported problems", config_for_task.remote_host);
                }

                let interval = Duration::from_secs(interval_minutes * 60);
                if let Ok(mut runs) = runs.lock() {
                    runs.next_run = Some(
                        (chrono::Local::now() + chrono::Duration::from_std(interval).unwrap_or_default())
                            .format("%Y-%m-%d %H:%M:%S")
                            .to_string(),
                    );
                }
                tokio::time::sleep(interval).await;
            }
        });

        *active = Some(ActiveSchedule { config, handle });
        info!("scheduled sync started every {interval_minutes} minutes");
        Ok(())
    }

    pub fn stop(&self) -> Result<(), String> {
        let mut active = self
            .active
            .lock()
            .map_err(|_| "Scheduler state is unavailable".to_string())?;
        if let Some(existing) = active.take() {
            existing.handle.abort();
        }
        if let Ok(mut runs) = self.runs.lock() {
            runs.last_run = None;
            runs.next_run = None;
        }
        info!("scheduled sync stopped");
        Ok(())
    }

    pub fn status(&self) -> Option<SyncScheduleStatus> {
        let active = self.active.lock().ok()?;
        let config = active.as_ref()?.config.clone();
        let runs = self.runs.lock().ok()?;
        Some(SyncScheduleStatus {
            source: config.source,
            destination: config.destination,
            remote_host: config.remote_host,
            interval_minutes: config.interval_minutes,
            last_run: runs.last_run.clone(),
            next_run: runs.next_run.clone(),
        })
    }

    pub fn is_running(&self) -> bool {
        self.active.lock().map(|active| active.is_some()).unwrap_or(false)
    }
}
