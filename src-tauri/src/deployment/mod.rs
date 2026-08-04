use serde::Serialize;
use std::path::{Path, PathBuf};
use tokio::fs;

pub mod meta;
pub mod templates;
pub mod vhosts;

use meta::{DeploymentMeta, MetaStore};

#[derive(Debug, Clone, Serialize)]
pub struct Deployment {
    pub name: String,
    pub path: String,
    pub url: String,
    pub network_url: String,
    pub modified: String,
    pub framework: String,
    pub tags: Vec<String>,
    pub pinned: bool,
    pub custom_domain: Option<String>,
    pub linked_db: Option<String>,
    pub vhost_enabled: bool,
    pub ssl_enabled: bool,
    pub last_opened: Option<String>,
    pub has_env: bool,
    pub has_composer: bool,
    pub has_package_json: bool,
    pub vhost_domain: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VhostInfo {
    pub domain: String,
    pub root: String,
    pub port: u16,
    pub ssl: bool,
}

fn deployment_root() -> PathBuf {
    crate::paths::xampp_root().join("htdocs")
}

fn validate_name(name: &str) -> Result<&str, String> {
    let name = name.trim();
    if name.is_empty() || name.len() > 80 {
        return Err("Deployment name must be between 1 and 80 characters".to_string());
    }
    if name == "."
        || name == ".."
        || !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
    {
        return Err(
            "Deployment names may contain only letters, numbers, hyphens, and underscores"
                .to_string(),
        );
    }
    Ok(name)
}

async fn ensure_deployment_root() -> Result<PathBuf, crate::error::AppError> {
    let root = deployment_root();
    fs::create_dir_all(&root)
        .await
        .map_err(|e| format!("Failed to create htdocs directory: {e}"))?;
    crate::paths::ensure_existing_path_in_xampp(&root).map_err(|e| e.into())
}

fn deployment_from_path(
    path: &Path,
    meta_store: &MetaStore,
) -> Result<Deployment, crate::error::AppError> {
    let metadata =
        std::fs::metadata(path).map_err(|e| format!("Failed to read deployment metadata: {e}"))?;
    let modified = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .and_then(|duration| chrono::DateTime::from_timestamp(duration.as_secs() as i64, 0))
        .map(|time| time.format("%Y-%m-%d %H:%M").to_string())
        .unwrap_or_else(|| "Unknown".to_string());
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "Invalid deployment name".to_string())?
        .to_string();
    let meta = meta_store.get(&name);
    let framework = meta
        .framework
        .clone()
        .unwrap_or_else(|| templates::detect_framework(path));
    let port = crate::paths::apache_port();
    let port_suffix = if port == 80 { String::new() } else { format!(":{}", port) };
    let ip = crate::paths::local_ip();

    let (url, network_url, vhost_domain) = if let Some(domain) = &meta.custom_domain {
        (
            format!("http://{domain}/"),
            format!("http://{domain}/"),
            Some(domain.clone()),
        )
    } else {
        (
            format!("http://localhost{port_suffix}/{name}/"),
            format!("http://{ip}{port_suffix}/{name}/"),
            None,
        )
    };

    let vhost_enabled = meta.vhost_enabled || vhosts::vhost_for_deployment(&name).is_some();
    let ssl_enabled = meta.ssl_enabled
        || crate::paths::ssl_crt_dir()
            .join(format!("{name}.crt"))
            .is_file();

    Ok(Deployment {
        url,
        network_url,
        name,
        path: path.to_string_lossy().to_string(),
        modified,
        framework,
        tags: meta.tags.clone(),
        pinned: meta.pinned,
        custom_domain: meta.custom_domain.clone(),
        linked_db: meta.linked_db.clone(),
        vhost_enabled,
        ssl_enabled,
        last_opened: meta.last_opened.clone(),
        has_env: path.join(".env").is_file(),
        has_composer: path.join("composer.json").is_file(),
        has_package_json: path.join("package.json").is_file(),
        vhost_domain,
    })
}

pub async fn list_deployments(
    app_data_dir: &Path,
) -> Result<Vec<Deployment>, crate::error::AppError> {
    let root = ensure_deployment_root().await?;
    let meta_store = MetaStore::load(&meta::meta_file(app_data_dir));
    let mut entries = fs::read_dir(root)
        .await
        .map_err(|e| format!("Failed to read htdocs: {e}"))?;
    let mut deployments = Vec::new();
    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|e| format!("Failed to read htdocs entry: {e}"))?
    {
        let path = entry.path();
        let file_type = entry
            .file_type()
            .await
            .map_err(|e| format!("Failed to read deployment type: {e}"))?;
        if file_type.is_dir() && !file_type.is_symlink() {
            deployments.push(deployment_from_path(&path, &meta_store)?);
        }
    }
    deployments.sort_by(|a, b| {
        b.pinned
            .cmp(&a.pinned)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    Ok(deployments)
}

pub async fn create_deployment(
    name: String,
    framework: String,
    app_data_dir: &Path,
) -> Result<Deployment, crate::error::AppError> {
    let name = validate_name(&name)?;
    let framework = if templates::FRAMEWORKS.contains(&framework.as_str()) {
        framework
    } else {
        "html".to_string()
    };
    let root = ensure_deployment_root().await?;
    let path = root.join(name);
    if path.exists() {
        return Err(format!("A deployment named '{name}' already exists").into());
    }
    fs::create_dir(&path)
        .await
        .map_err(|e| format!("Failed to create deployment: {e}"))?;

    let files = templates::template_files(name, &framework);
    let write_result: Result<(), std::io::Error> = async {
        for (relative, content) in files {
            let target = path.join(&relative);
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).await?;
            }
            fs::write(target, content).await?;
        }
        Ok(())
    }
    .await;
    if let Err(error) = write_result {
        let _ = fs::remove_dir_all(&path).await;
        return Err(format!("Failed to create starter project: {error}").into());
    }

    // Record framework + auto-provision DB if requested via linked_db
    let mut meta_store = MetaStore::load(&meta::meta_file(app_data_dir));
    meta_store.set(
        name.to_string(),
        DeploymentMeta {
            framework: Some(framework.clone()),
            last_opened: Some(now_string()),
            ..Default::default()
        },
    );
    let _ = meta_store.save(&meta::meta_file(app_data_dir));

    deployment_from_path(&path, &meta_store)
}

pub async fn import_deployment(
    name: String,
    source_path: String,
    framework: Option<String>,
    app_data_dir: &Path,
) -> Result<Deployment, crate::error::AppError> {
    let name = validate_name(&name)?.to_string();
    let root = ensure_deployment_root().await?;
    let target = root.join(&name);
    if target.exists() {
        return Err(format!("A deployment named '{name}' already exists").into());
    }
    let source = std::fs::canonicalize(source_path)
        .map_err(|_| "The selected project folder is no longer available".to_string())?;
    let source_metadata = std::fs::symlink_metadata(&source)
        .map_err(|e| format!("Failed to read selected project: {e}"))?;
    if source_metadata.file_type().is_symlink() || !source_metadata.is_dir() {
        return Err("Select a normal project folder to import".to_string().into());
    }
    if target.starts_with(&source) {
        return Err("Select a project folder, not the XAMPP or htdocs parent folder".to_string().into());
    }
    let source_for_copy = source.clone();
    let target_for_copy = target.clone();
    let copy_result =
        tokio::task::spawn_blocking(move || copy_project_tree(&source_for_copy, &target_for_copy))
            .await
            .map_err(|e| format!("Project import task failed: {e}"))?;
    if let Err(error) = copy_result {
        let _ = fs::remove_dir_all(&target).await;
        return Err(error.into());
    }
    let detected = framework
        .clone()
        .unwrap_or_else(|| templates::detect_framework(&target));
    let mut meta_store = MetaStore::load(&meta::meta_file(app_data_dir));
    meta_store.set(
        name.clone(),
        DeploymentMeta {
            framework: Some(detected),
            last_opened: Some(now_string()),
            ..Default::default()
        },
    );
    let _ = meta_store.save(&meta::meta_file(app_data_dir));
    deployment_from_path(&target, &meta_store)
}

pub async fn delete_deployment(name: String, app_data_dir: &Path) -> Result<(), crate::error::AppError> {
    let name = validate_name(&name)?;
    let root = ensure_deployment_root().await?;
    let path = root.join(name);
    let metadata = fs::symlink_metadata(&path)
        .await
        .map_err(|e| format!("Deployment not found: {e}"))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err("Deployment must be a normal directory inside htdocs".to_string().into());
    }
    let canonical = crate::paths::ensure_existing_path_in_xampp(&path)?;
    if canonical.parent() != Some(root.as_path()) {
        return Err("Refusing to delete a path outside htdocs".to_string().into());
    }
    let canonical_for_recycle = canonical.clone();
    tokio::task::spawn_blocking(move || crate::recycle::recycle_path(&canonical_for_recycle))
        .await
        .map_err(|e| format!("Delete task failed: {e}"))??;
    let mut meta_store = MetaStore::load(&meta::meta_file(app_data_dir));
    meta_store.remove(name);
    let _ = meta_store.save(&meta::meta_file(app_data_dir));
    Ok(())
}

pub async fn update_meta(
    name: String,
    updates: DeploymentUpdate,
    app_data_dir: &Path,
) -> Result<Deployment, crate::error::AppError> {
    let name = validate_name(&name)?;
    let root = ensure_deployment_root().await?;
    let path = root.join(name);
    if !path.is_dir() {
        return Err(format!("Deployment '{name}' not found").into());
    }
    let mut meta_store = MetaStore::load(&meta::meta_file(app_data_dir));
    meta_store.update(name, |meta| {
        if let Some(framework) = &updates.framework {
            meta.framework = Some(framework.clone());
        }
        if let Some(tags) = &updates.tags {
            meta.tags = tags.clone();
        }
        if let Some(pinned) = updates.pinned {
            meta.pinned = pinned;
        }
        if let Some(linked_db) = &updates.linked_db {
            meta.linked_db = Some(linked_db.clone());
        }
        if updates.last_opened {
            meta.last_opened = Some(now_string());
        }
    });
    let _ = meta_store.save(&meta::meta_file(app_data_dir));
    deployment_from_path(&path, &meta_store)
}

pub async fn duplicate_deployment(
    name: String,
    new_name: String,
    app_data_dir: &Path,
) -> Result<Deployment, crate::error::AppError> {
    let name = validate_name(&name)?;
    let new_name = validate_name(&new_name)?.to_string();
    let root = ensure_deployment_root().await?;
    let source = root.join(name);
    if !source.is_dir() {
        return Err(format!("Deployment '{name}' not found").into());
    }
    let target = root.join(&new_name);
    if target.exists() {
        return Err(format!("A deployment named '{new_name}' already exists").into());
    }
    let source_for_copy = source.clone();
    let target_for_copy = target.clone();
    let copy_result =
        tokio::task::spawn_blocking(move || copy_project_tree(&source_for_copy, &target_for_copy))
            .await
            .map_err(|e| format!("Duplicate task failed: {e}"))?;
    if let Err(error) = copy_result {
        let _ = fs::remove_dir_all(&target).await;
        return Err(error.into());
    }
    let mut meta_store = MetaStore::load(&meta::meta_file(app_data_dir));
    let mut new_meta = meta_store.get(name);
    new_meta.custom_domain = None;
    new_meta.vhost_enabled = false;
    new_meta.ssl_enabled = false;
    new_meta.last_opened = Some(now_string());
    meta_store.set(new_name.clone(), new_meta);
    let _ = meta_store.save(&meta::meta_file(app_data_dir));
    deployment_from_path(&target, &meta_store)
}

pub async fn git_import(
    name: String,
    repo_url: String,
    framework: Option<String>,
    app_data_dir: &Path,
) -> Result<Deployment, crate::error::AppError> {
    let name = validate_name(&name)?.to_string();
    let root = ensure_deployment_root().await?;
    let target = root.join(&name);
    if target.exists() {
        return Err(format!("A deployment named '{name}' already exists").into());
    }
    let output = tokio::process::Command::new("git")
        .args(["clone", "--depth", "1"])
        .arg(&repo_url)
        .arg(&target)
        .output()
        .await
        .map_err(|e| format!("Git is not available: {e}"))?;
    if !output.status.success() {
        let _ = fs::remove_dir_all(&target).await;
        return Err(format!(
            "Git clone failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )
        .into());
    }
    let detected = framework
        .clone()
        .unwrap_or_else(|| templates::detect_framework(&target));
    let mut meta_store = MetaStore::load(&meta::meta_file(app_data_dir));
    meta_store.set(
        name.clone(),
        DeploymentMeta {
            framework: Some(detected),
            last_opened: Some(now_string()),
            ..Default::default()
        },
    );
    let _ = meta_store.save(&meta::meta_file(app_data_dir));
    deployment_from_path(&target, &meta_store)
}

pub async fn set_custom_domain(
    name: String,
    domain: Option<String>,
    app_data_dir: &Path,
) -> Result<Deployment, crate::error::AppError> {
    let name = validate_name(&name)?;
    let root = ensure_deployment_root().await?;
    let path = root.join(name);
    if !path.is_dir() {
        return Err(format!("Deployment '{name}' not found").into());
    }
    let mut meta_store = MetaStore::load(&meta::meta_file(app_data_dir));
    let old_domain = meta_store.get(name).custom_domain.clone();
    match &domain {
        Some(d) => {
            let d = d.trim();
            if d.is_empty() {
                return Err("Enter a domain such as myapp.test".to_string().into());
            }
            // ensure the vhost exists and update the hosts entry
            let vhost = vhosts::set_vhost(name, Some(d), false)?;
            vhosts::set_hosts_entry(Some(&vhost.domain)).ok();
            if let Some(old) = &old_domain {
                if old != &vhost.domain {
                    vhosts::remove_hosts_entry(old).ok();
                }
            }
            meta_store.update(name, |meta| {
                meta.custom_domain = Some(vhost.domain);
                meta.vhost_enabled = true;
            });
        }
        None => {
            if let Some(old) = &old_domain {
                vhosts::remove_hosts_entry(old).ok();
            }
            vhosts::set_vhost(name, None, false).ok();
            meta_store.update(name, |meta| {
                meta.custom_domain = None;
                meta.vhost_enabled = false;
            });
        }
    }
    let _ = meta_store.save(&meta::meta_file(app_data_dir));
    deployment_from_path(&path, &meta_store)
}

pub async fn toggle_vhost(
    name: String,
    enabled: bool,
    app_data_dir: &Path,
) -> Result<Deployment, crate::error::AppError> {
    let name = validate_name(&name)?;
    let root = ensure_deployment_root().await?;
    let path = root.join(name);
    if !path.is_dir() {
        return Err(format!("Deployment '{name}' not found").into());
    }
    let mut meta_store = MetaStore::load(&meta::meta_file(app_data_dir));
    if enabled {
        let domain = meta_store
            .get(name)
            .custom_domain
            .unwrap_or_else(|| format!("{name}.test"));
        let vhost = vhosts::set_vhost(name, Some(&domain), false)?;
        vhosts::set_hosts_entry(Some(&vhost.domain)).ok();
        meta_store.update(name, |meta| {
            meta.vhost_enabled = true;
            meta.custom_domain = Some(vhost.domain);
        });
    } else {
        let meta = meta_store.get(name);
        if let Some(domain) = meta.custom_domain {
            vhosts::remove_hosts_entry(&domain).ok();
        }
        vhosts::set_vhost(name, None, false).ok();
        meta_store.update(name, |meta| {
            meta.vhost_enabled = false;
            meta.custom_domain = None;
        });
    }
    let _ = meta_store.save(&meta::meta_file(app_data_dir));
    deployment_from_path(&path, &meta_store)
}

pub async fn enable_ssl(
    name: String,
    app_data_dir: &Path,
) -> Result<Deployment, crate::error::AppError> {
    let name = validate_name(&name)?;
    let root = ensure_deployment_root().await?;
    let path = root.join(name);
    if !path.is_dir() {
        return Err(format!("Deployment '{name}' not found").into());
    }
    let domain = format!("{name}.test");
    // Generate a self-signed certificate for this deployment
    let _ = crate::ssl_manager::generate_self_signed(domain.clone(), 825).await?;
    // Add an SSL vhost entry
    vhosts::set_vhost(name, Some(&domain), true)?;
    let mut meta_store = MetaStore::load(&meta::meta_file(app_data_dir));
    meta_store.update(name, |meta| {
        meta.ssl_enabled = true;
        meta.vhost_enabled = true;
        meta.custom_domain = Some(domain);
    });
    let _ = meta_store.save(&meta::meta_file(app_data_dir));
    deployment_from_path(&path, &meta_store)
}

pub async fn read_env(name: String) -> Result<String, crate::error::AppError> {
    let name = validate_name(&name)?;
    let root = ensure_deployment_root().await?;
    let path = root.join(name).join(".env");
    if !path.exists() {
        return Err(format!("No .env file found for '{name}'").into());
    }
    Ok(fs::read_to_string(&path)
        .await
        .map_err(|e| format!("Failed to read .env: {e}"))?)
}

pub async fn write_env(name: String, content: String) -> Result<(), crate::error::AppError> {
    let name = validate_name(&name)?;
    let root = ensure_deployment_root().await?;
    let path = root.join(name).join(".env");
    if content.contains('\0') {
        return Err("Invalid content".to_string().into());
    }
    fs::write(&path, content)
        .await
        .map_err(|e| format!("Failed to write .env: {e}"))?;
    Ok(())
}

pub async fn run_dependency_command(
    name: String,
    tool: String,
    action: String,
) -> Result<RunOutput, crate::error::AppError> {
    let name = validate_name(&name)?;
    let root = ensure_deployment_root().await?;
    let dir = root.join(name);
    if !dir.is_dir() {
        return Err(format!("Deployment '{name}' not found").into());
    }
    let (program, args): (&str, Vec<String>) = match (tool.as_str(), action.as_str()) {
        ("composer", "install") => ("composer", vec!["install".into(), "--no-interaction".into()]),
        ("composer", "update") => ("composer", vec!["update".into(), "--no-interaction".into()]),
        ("composer", "outdated") => ("composer", vec!["outdated".into(), "--direct".into()]),
        ("npm", "install") => ("npm", vec!["install".into()]),
        ("npm", "build") => ("npm", vec!["run".into(), "build".into()]),
        ("npm", "outdated") => ("npm", vec!["outdated".into()]),
        ("npm", "dev") => ("npm", vec!["run".into(), "dev".into()]),
        _ => return Err(format!("Unsupported tool/action: {tool}/{action}").into()),
    };
    let output = tokio::process::Command::new(program)
        .args(args)
        .current_dir(&dir)
        .output()
        .await
        .map_err(|e| format!("{tool} is not available: {e}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let combined = if stdout.is_empty() { stderr } else { stdout };
    Ok(RunOutput {
        success: output.status.success(),
        output: combined,
    })
}

pub async fn git_info(name: String) -> Result<GitInfo, crate::error::AppError> {
    let name = validate_name(&name)?;
    let root = ensure_deployment_root().await?;
    let dir = root.join(name);
    if !dir.join(".git").is_dir() {
        return Ok(GitInfo {
            is_git: false,
            branch: None,
            last_commit: None,
            dirty: false,
        });
    }
    let branch = git_run(&dir, &["rev-parse", "--abbrev-ref", "HEAD"]).await;
    let last_commit = git_run(&dir, &["log", "-1", "--format=%h %s"]).await;
    let status = git_run(&dir, &["status", "--porcelain"]).await;
    let has_git = !(branch.is_empty() && last_commit.is_empty() && status.is_empty());
    Ok(GitInfo {
        is_git: has_git,
        branch: (!branch.is_empty()).then_some(branch),
        last_commit: (!last_commit.is_empty()).then_some(last_commit),
        dirty: !status.is_empty(),
    })
}

async fn git_run(dir: &Path, args: &[&str]) -> String {
    tokio::process::Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .await
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

pub async fn list_backups(
    name: Option<String>,
    app_data_dir: &Path,
) -> Result<Vec<BackupInfo>, crate::error::AppError> {
    let backups_dir = app_data_dir.join("deployment-backups");
    let Ok(mut entries) = fs::read_dir(&backups_dir).await else {
        return Ok(Vec::new());
    };
    let mut backups = Vec::new();
    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|e| format!("Failed to read backups: {e}"))?
    {
        let p = entry.path();
        if p.extension().is_some_and(|e| e == "zip") {
            let filename = p
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            // Filename format: {deployment}-{timestamp}
            if let Some(prefix) = filename.rfind('-') {
                let dep = &filename[..prefix];
                if name.as_deref().is_some_and(|n| n == dep) || name.is_none() {
                    let size = std::fs::metadata(&p).map(|m| m.len()).unwrap_or(0);
                    backups.push(BackupInfo {
                        deployment: dep.to_string(),
                        timestamp: filename[prefix + 1..].to_string(),
                        path: p.to_string_lossy().to_string(),
                        size,
                    });
                }
            }
        }
    }
    backups.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    Ok(backups)
}

pub async fn restore_backup(
    name: String,
    backup_path: String,
    app_data_dir: &Path,
) -> Result<Deployment, crate::error::AppError> {
    let name = validate_name(&name)?;
    let root = ensure_deployment_root().await?;
    let target = root.join(name);
    if target.exists() {
        return Err(format!("A deployment named '{name}' already exists").into());
    }
    let backups_dir = app_data_dir.join("deployment-backups");
    let canonical_backup = std::fs::canonicalize(Path::new(&backup_path))
        .map_err(|_| "Backup file is no longer available".to_string())?;
    if !canonical_backup.starts_with(&backups_dir) || canonical_backup.extension().map(|e| e != "zip").unwrap_or(true) {
        return Err("Invalid backup file".to_string().into());
    }
    let canonical_backup2 = canonical_backup.clone();
    let target2 = target.clone();
    tokio::task::spawn_blocking(move || extract_zip(&canonical_backup2, &target2))
        .await
        .map_err(|e| format!("Restore task failed: {e}"))??;
    let detected = templates::detect_framework(&target);
    let mut meta_store = MetaStore::load(&meta::meta_file(app_data_dir));
    meta_store.set(
        name.to_string(),
        DeploymentMeta {
            framework: Some(detected),
            last_opened: Some(now_string()),
            ..Default::default()
        },
    );
    let _ = meta_store.save(&meta::meta_file(app_data_dir));
    deployment_from_path(&target, &meta_store)
}

pub async fn open_deployment(
    name: String,
    app_data_dir: &Path,
) -> Result<Deployment, crate::error::AppError> {
    let name = validate_name(&name)?;
    let root = ensure_deployment_root().await?;
    let path = root.join(name);
    if !path.is_dir() {
        return Err(format!("Deployment '{name}' not found").into());
    }
    let mut meta_store = MetaStore::load(&meta::meta_file(app_data_dir));
    meta_store.update(name, |meta| {
        meta.last_opened = Some(now_string());
    });
    let _ = meta_store.save(&meta::meta_file(app_data_dir));
    deployment_from_path(&path, &meta_store)
}

fn copy_project_tree(source: &Path, destination: &Path) -> Result<(), String> {
    const MAX_FILES: usize = 10_000;
    const MAX_BYTES: u64 = 500 * 1024 * 1024;
    fn copy_directory(
        source: &Path,
        destination: &Path,
        files: &mut usize,
        bytes: &mut u64,
    ) -> Result<(), String> {
        std::fs::create_dir_all(destination)
            .map_err(|e| format!("Failed to create project directory: {e}"))?;
        for entry in
            std::fs::read_dir(source).map_err(|e| format!("Failed to read project folder: {e}"))?
        {
            let entry = entry.map_err(|e| format!("Failed to read project entry: {e}"))?;
            let source_path = entry.path();
            let destination_path = destination.join(entry.file_name());
            let file_type = entry
                .file_type()
                .map_err(|e| format!("Failed to inspect project entry: {e}"))?;
            if file_type.is_symlink() {
                return Err("Project imports cannot include symbolic links".to_string());
            }
            if file_type.is_dir() {
                copy_directory(&source_path, &destination_path, files, bytes)?;
            } else if file_type.is_file() {
                *files += 1;
                *bytes += entry
                    .metadata()
                    .map_err(|e| format!("Failed to inspect project file: {e}"))?
                    .len();
                if *files > MAX_FILES || *bytes > MAX_BYTES {
                    return Err(
                        "Project is too large to import (maximum 10,000 files or 500 MB)"
                            .to_string(),
                    );
                }
                std::fs::copy(&source_path, &destination_path)
                    .map_err(|e| format!("Failed to copy project file: {e}"))?;
            }
        }
        Ok(())
    }
    let mut files = 0;
    let mut bytes = 0;
    copy_directory(source, destination, &mut files, &mut bytes)
}

fn extract_zip(zip_path: &Path, target: &Path) -> Result<(), String> {
    let file = std::fs::File::open(zip_path)
        .map_err(|e| format!("Failed to open backup: {e}"))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| format!("Failed to read backup archive: {e}"))?;
    std::fs::create_dir_all(target)
        .map_err(|e| format!("Failed to create deployment folder: {e}"))?;
    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("Failed to read backup entry: {e}"))?;
        let entry_name = entry.name().to_string();
        // Prevent path traversal
        let entry_path = entry
            .name()
            .split('/')
            .collect::<Vec<_>>()
            .join(std::path::MAIN_SEPARATOR_STR);
        let entry_path = entry_path.trim_start_matches(std::path::MAIN_SEPARATOR_STR);
        let out = target.join(entry_path);
        if !out.starts_with(target) {
            return Err("Backup contains an unsafe path".to_string());
        }
        if entry.is_dir() {
            std::fs::create_dir_all(&out)
                .map_err(|e| format!("Failed to create folder: {e}"))?;
        } else {
            if let Some(parent) = out.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create folder: {e}"))?;
            }
            let mut writer = std::fs::File::create(&out)
                .map_err(|e| format!("Failed to create file: {e}"))?;
            std::io::copy(&mut entry, &mut writer)
                .map_err(|e| format!("Failed to extract file: {e}"))?;
        }
        let _ = entry_name;
    }
    Ok(())
}

fn now_string() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

#[derive(Debug, Clone, Serialize)]
pub struct RunOutput {
    pub success: bool,
    pub output: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct GitInfo {
    pub is_git: bool,
    pub branch: Option<String>,
    pub last_commit: Option<String>,
    pub dirty: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct BackupInfo {
    pub deployment: String,
    pub timestamp: String,
    pub path: String,
    pub size: u64,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct DeploymentUpdate {
    pub framework: Option<String>,
    pub tags: Option<Vec<String>>,
    pub pinned: Option<bool>,
    pub linked_db: Option<String>,
    pub last_opened: bool,
}

// Keep the backup function for compatibility (used by old command registration)
pub async fn backup_deployment(name: String, app_data_dir: PathBuf) -> Result<PathBuf, crate::error::AppError> {
    let name = validate_name(&name)?;
    let root = ensure_deployment_root().await?;
    let path = root.join(name);
    let metadata = fs::symlink_metadata(&path)
        .await
        .map_err(|e| format!("Deployment not found: {e}"))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err("Deployment must be a normal directory inside htdocs".to_string().into());
    }
    let canonical = crate::paths::ensure_existing_path_in_xampp(&path)?;
    if canonical.parent() != Some(root.as_path()) {
        return Err("Refusing to back up a path outside htdocs".to_string().into());
    }
    let backups_dir = app_data_dir.join("deployment-backups");
    fs::create_dir_all(&backups_dir)
        .await
        .map_err(|e| format!("Failed to create backups folder: {e}"))?;
    let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
    let destination = backups_dir.join(format!("{name}-{timestamp}.zip"));
    let source_for_zip = canonical.clone();
    let destination_for_zip = destination.clone();
    tokio::task::spawn_blocking(move || zip_directory_tree(&source_for_zip, &destination_for_zip))
        .await
        .map_err(|e| format!("Backup task failed: {e}"))??;
    Ok(destination)
}

fn zip_directory_tree(source: &Path, destination: &Path) -> Result<(), String> {
    const MAX_FILES: usize = 10_000;
    const MAX_BYTES: u64 = 500 * 1024 * 1024;

    let file = std::fs::File::create(destination)
        .map_err(|e| format!("Failed to create backup file: {e}"))?;
    let mut writer = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    fn walk(
        writer: &mut zip::ZipWriter<std::fs::File>,
        directory: &Path,
        base: &str,
        options: zip::write::SimpleFileOptions,
        files: &mut usize,
        bytes: &mut u64,
    ) -> Result<(), String> {
        for entry in std::fs::read_dir(directory)
            .map_err(|e| format!("Failed to read deployment folder: {e}"))?
        {
            let entry =
                entry.map_err(|e| format!("Failed to read deployment entry: {e}"))?;
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            let relative = if base.is_empty() {
                name.clone()
            } else {
                format!("{base}/{name}")
            };
            let file_type = entry
                .file_type()
                .map_err(|e| format!("Failed to inspect deployment entry: {e}"))?;
            if file_type.is_symlink() {
                return Err("Backups cannot include symbolic links".to_string());
            }
            if file_type.is_dir() {
                writer
                    .add_directory(format!("{relative}/"), options)
                    .map_err(|e| format!("Failed to archive folder: {e}"))?;
                walk(writer, &path, &relative, options, files, bytes)?;
            } else if file_type.is_file() {
                *files += 1;
                *bytes += entry
                    .metadata()
                    .map_err(|e| format!("Failed to inspect deployment file: {e}"))?
                    .len();
                if *files > MAX_FILES || *bytes > MAX_BYTES {
                    return Err(
                        "Deployment is too large to back up (maximum 10,000 files or 500 MB)"
                            .to_string(),
                    );
                }
                let mut reader = std::fs::File::open(&path)
                    .map_err(|e| format!("Failed to open deployment file: {e}"))?;
                writer
                    .start_file(relative, options)
                    .map_err(|e| format!("Failed to start archive entry: {e}"))?;
                std::io::copy(&mut reader, writer)
                    .map_err(|e| format!("Failed to archive deployment file: {e}"))?;
            }
        }
        Ok(())
    }

    let mut files = 0;
    let mut bytes = 0;
    walk(&mut writer, source, "", options, &mut files, &mut bytes)?;
    writer
        .finish()
        .map_err(|e| format!("Failed to finalize backup: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_name;

    #[test]
    fn accepts_safe_deployment_names() {
        assert_eq!(validate_name("my_app-2"), Ok("my_app-2"));
    }

    #[test]
    fn rejects_path_traversal() {
        assert!(validate_name("../outside").is_err());
    }

    #[test]
    fn rejects_special_chars() {
        assert!(validate_name("my app").is_err());
    }
}
