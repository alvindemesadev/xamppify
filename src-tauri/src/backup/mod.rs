use chrono::Local;
use serde::{Deserialize, Serialize};
use tokio::fs;

const BACKUP_DIR: &str = "backups";

fn backup_root() -> Result<std::path::PathBuf, String> {
    let root = crate::paths::xampp_root();
    let dir = root.join(BACKUP_DIR);
    Ok(dir)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupInfo {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub created: String,
}

pub async fn create_backup(name: Option<String>) -> Result<BackupInfo, String> {
    let root = crate::paths::xampp_root();
    let backup_dir = backup_root()?;
    fs::create_dir_all(&backup_dir)
        .await
        .map_err(|e| format!("Failed to create backup dir: {}", e))?;

    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let name = name.unwrap_or_else(|| format!("backup_{}", timestamp));
    let safe_name: String = name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '_' || c == '-' { c } else { '_' })
        .collect();
    let backup_path = backup_dir.join(format!("{}.zip", safe_name));

    let htdocs = root.join("htdocs");
    if htdocs.exists() {
        let mut command = tokio::process::Command::new("powershell");
        command
            .arg("-NoProfile")
            .arg("-Command")
            .arg(format!(
                "Compress-Archive -Path '{}' -DestinationPath '{}' -Force",
                htdocs.display(),
                backup_path.display()
            ));
        crate::paths::hide_console_for_tokio_command(&mut command);
        let output = command
            .output()
            .await
            .map_err(|e| format!("Failed to create backup: {}", e))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Backup failed: {}", stderr));
        }
    }

    let metadata = fs::metadata(&backup_path).await.map_err(|_| ());
    let size = metadata.map(|m| m.len()).unwrap_or(0);

    Ok(BackupInfo {
        name: safe_name,
        path: backup_path.to_string_lossy().to_string(),
        size,
        created: timestamp.to_string(),
    })
}

pub async fn list_backups() -> Result<Vec<BackupInfo>, String> {
    let backup_dir = backup_root()?;
    if !backup_dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries = fs::read_dir(&backup_dir)
        .await
        .map_err(|e| format!("Failed to read backups: {}", e))?;

    let mut backups = Vec::new();
    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|e| format!("Failed to read entry: {}", e))?
    {
        let path = entry.path();
        if path.extension().map(|e| e == "zip").unwrap_or(false) {
            let meta = fs::metadata(&path).await.ok();
            let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
            let created = meta
                .and_then(|m| m.modified().ok())
                .map(|t| {
                    let dur = t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
                    let secs = dur.as_secs() as i64;
                    chrono::DateTime::from_timestamp(secs, 0)
                        .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_default()
                })
                .unwrap_or_default();
            backups.push(BackupInfo {
                name: entry.file_name().to_string_lossy().to_string(),
                path: path.to_string_lossy().to_string(),
                size,
                created,
            });
        }
    }

    backups.sort_by(|a, b| b.created.cmp(&a.created));
    Ok(backups)
}

pub async fn delete_backup(name: &str) -> Result<(), String> {
    let backup_dir = backup_root()?;
    let path = backup_dir.join(name);
    if path.exists() {
        fs::remove_file(&path)
            .await
            .map_err(|e| format!("Failed to delete backup: {}", e))?;
    }
    Ok(())
}

pub async fn dump_mysql() -> Result<String, String> {
    let mysqldump = crate::paths::xampp_root().join("mysql").join("bin").join("mysqldump.exe");
    if !mysqldump.exists() {
        return Err("mysqldump not found".to_string());
    }

    let backup_dir = backup_root()?;
    fs::create_dir_all(&backup_dir).await.map_err(|e| format!("Failed to create backup dir: {}", e))?;

    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let out_path = backup_dir.join(format!("mysql_dump_{}.sql", timestamp));

    let mut command = tokio::process::Command::new(
        crate::paths::path_for_external_command(&mysqldump)
    );
    command
        .arg("--all-databases")
        .arg("-u").arg("root")
        .arg("-r")
        .arg(crate::paths::path_for_external_command(&out_path));
    crate::paths::hide_console_for_tokio_command(&mut command);

    let output = command
        .output()
        .await
        .map_err(|e| format!("Failed to run mysqldump: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("mysqldump failed: {}", stderr));
    }

    Ok(out_path.to_string_lossy().to_string())
}
