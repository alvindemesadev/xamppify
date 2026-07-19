use serde::Serialize;
use std::path::{Path, PathBuf};
use tokio::fs;

#[derive(Debug, Clone, Serialize)]
pub struct Deployment {
    pub name: String,
    pub path: String,
    pub url: String,
    pub modified: String,
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

async fn ensure_deployment_root() -> Result<PathBuf, String> {
    let root = deployment_root();
    fs::create_dir_all(&root)
        .await
        .map_err(|e| format!("Failed to create htdocs directory: {e}"))?;
    crate::paths::ensure_existing_path_in_xampp(&root)
}

fn deployment_from_path(path: &Path) -> Result<Deployment, String> {
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
    Ok(Deployment {
        url: format!("http://localhost/{name}/"),
        name,
        path: path.to_string_lossy().to_string(),
        modified,
    })
}

pub async fn list_deployments() -> Result<Vec<Deployment>, String> {
    let root = ensure_deployment_root().await?;
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
            deployments.push(deployment_from_path(&path)?);
        }
    }
    deployments.sort_by_key(|deployment| deployment.name.to_lowercase());
    Ok(deployments)
}

pub async fn create_deployment(name: String, template: String) -> Result<Deployment, String> {
    let name = validate_name(&name)?;
    if !matches!(template.as_str(), "html" | "php") {
        return Err("Choose either the HTML or PHP starter template".to_string());
    }
    let root = ensure_deployment_root().await?;
    let path = root.join(name);
    if path.exists() {
        return Err(format!("A deployment named '{name}' already exists"));
    }
    fs::create_dir(&path)
        .await
        .map_err(|e| format!("Failed to create deployment: {e}"))?;
    let assets = path.join("assets");
    let title = name.replace(['-', '_'], " ");
    let index_name = if template == "php" {
        "index.php"
    } else {
        "index.html"
    };
    let page = format!(
        "<!doctype html>\n<html lang=\"en\">\n<head>\n  <meta charset=\"utf-8\">\n  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n  <title>{title}</title>\n  <link rel=\"stylesheet\" href=\"assets/styles.css\">\n</head>\n<body>\n  <main class=\"hero\">\n    <p class=\"eyebrow\">Local XAMPP deployment</p>\n    <h1>{title}</h1>\n    <p>Edit <code>{index_name}</code>, <code>assets/styles.css</code>, and <code>assets/app.js</code> from Xamppify.</p>\n    <button id=\"hello-button\">Test JavaScript</button>\n  </main>\n  <script src=\"assets/app.js\"></script>\n</body>\n</html>\n"
    );
    let stylesheet = "* { box-sizing: border-box; }\nbody { margin: 0; min-height: 100vh; display: grid; place-items: center; background: #0f172a; color: #e2e8f0; font-family: system-ui, sans-serif; }\n.hero { max-width: 42rem; padding: 3rem; border: 1px solid #334155; border-radius: 1rem; background: #172033; }\n.eyebrow { color: #38bdf8; font-weight: 700; text-transform: uppercase; letter-spacing: .08em; font-size: .75rem; }\nh1 { margin: .25rem 0 1rem; font-size: clamp(2rem, 8vw, 4rem); }\nbutton { border: 0; border-radius: .5rem; padding: .75rem 1rem; background: #38bdf8; color: #082f49; font-weight: 700; cursor: pointer; }\n";
    let script = "document.querySelector('#hello-button')?.addEventListener('click', () => alert('Your deployment is working.'));\n";
    let write_result: Result<(), std::io::Error> = async {
        fs::create_dir(&assets).await?;
        fs::write(path.join(index_name), page).await?;
        fs::write(assets.join("styles.css"), stylesheet).await?;
        fs::write(assets.join("app.js"), script).await?;
        fs::write(path.join(".gitignore"), ".DS_Store\nThumbs.db\n").await
    }
    .await;
    if let Err(error) = write_result {
        let _ = fs::remove_dir(&path).await;
        return Err(format!("Failed to create starter project: {error}"));
    }
    deployment_from_path(&path)
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

pub async fn import_deployment(name: String, source_path: String) -> Result<Deployment, String> {
    let name = validate_name(&name)?.to_string();
    let root = ensure_deployment_root().await?;
    let target = root.join(&name);
    if target.exists() {
        return Err(format!("A deployment named '{name}' already exists"));
    }
    let source = std::fs::canonicalize(source_path)
        .map_err(|_| "The selected project folder is no longer available".to_string())?;
    let source_metadata = std::fs::symlink_metadata(&source)
        .map_err(|e| format!("Failed to read selected project: {e}"))?;
    if source_metadata.file_type().is_symlink() || !source_metadata.is_dir() {
        return Err("Select a normal project folder to import".to_string());
    }
    if target.starts_with(&source) {
        return Err("Select a project folder, not the XAMPP or htdocs parent folder".to_string());
    }
    let source_for_copy = source.clone();
    let target_for_copy = target.clone();
    let copy_result =
        tokio::task::spawn_blocking(move || copy_project_tree(&source_for_copy, &target_for_copy))
            .await
            .map_err(|e| format!("Project import task failed: {e}"))?;
    if let Err(error) = copy_result {
        let _ = fs::remove_dir_all(&target).await;
        return Err(error);
    }
    deployment_from_path(&target)
}

pub async fn delete_deployment(name: String) -> Result<(), String> {
    let name = validate_name(&name)?;
    let root = ensure_deployment_root().await?;
    let path = root.join(name);
    let metadata = fs::symlink_metadata(&path)
        .await
        .map_err(|e| format!("Deployment not found: {e}"))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err("Deployment must be a normal directory inside htdocs".to_string());
    }
    let canonical = crate::paths::ensure_existing_path_in_xampp(&path)?;
    if canonical.parent() != Some(root.as_path()) {
        return Err("Refusing to delete a path outside htdocs".to_string());
    }
    fs::remove_dir_all(canonical)
        .await
        .map_err(|e| format!("Failed to remove deployment: {e}"))
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
}
