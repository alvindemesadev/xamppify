use std::path::{Path, PathBuf};
use tokio::fs;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: String,
}

pub async fn list_directory(dir: &str) -> Result<Vec<FileEntry>, String> {
    let path = crate::paths::ensure_existing_path_in_xampp(Path::new(dir))?;
    if !path.is_dir() {
        return Err(format!("Not a directory: {}", dir));
    }

    let mut entries = Vec::new();
    let mut read_dir = fs::read_dir(&path)
        .await
        .map_err(|e| format!("Failed to read directory: {}", e))?;

    while let Some(entry) = read_dir
        .next_entry()
        .await
        .map_err(|e| format!("Failed to read entry: {}", e))?
    {
        let metadata = entry
            .metadata()
            .await
            .map_err(|e| format!("Failed to read metadata: {}", e))?;

        let modified = metadata
            .modified()
            .ok()
            .map(|t| {
                let dur = t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
                let secs = dur.as_secs();
                let dt = chrono::DateTime::from_timestamp(secs as i64, 0)
                    .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_default();
                dt
            })
            .unwrap_or_default();

        entries.push(FileEntry {
            name: entry.file_name().to_string_lossy().to_string(),
            path: entry.path().to_string_lossy().to_string(),
            is_dir: metadata.is_dir(),
            size: metadata.len(),
            modified,
        });
    }

    entries.sort_by(|a, b| {
        if a.is_dir != b.is_dir {
            b.is_dir.cmp(&a.is_dir)
        } else {
            a.name.cmp(&b.name)
        }
    });

    Ok(entries)
}

pub async fn read_file(path: &str) -> Result<String, String> {
    let path = crate::paths::ensure_existing_path_in_xampp(Path::new(path))?;
    if path.is_dir() {
        return Err("Cannot read a directory as a file".to_string());
    }
    let content = fs::read_to_string(path)
        .await
        .map_err(|e| format!("Failed to read file: {}", e))?;
    Ok(content)
}

pub async fn write_file(path: &str, content: &str) -> Result<(), String> {
    let path = crate::paths::ensure_writable_path_in_xampp(Path::new(path))?;
    if path.is_dir() {
        return Err("Cannot write to a directory".to_string());
    }
    fs::write(path, content)
        .await
        .map_err(|e| format!("Failed to write file: {}", e))
}

fn valid_child_name(name: &str) -> Result<&str, String> {
    let name = name.trim();
    if name.is_empty() || name == "." || name == ".." || name.contains(['/', '\\']) {
        return Err("Enter a valid file or folder name".to_string());
    }
    Ok(name)
}

fn existing_directory(path: &str) -> Result<PathBuf, String> {
    let path = crate::paths::ensure_existing_path_in_xampp(Path::new(path))?;
    if !path.is_dir() {
        return Err("Choose a folder inside XAMPP".to_string());
    }
    Ok(path)
}

pub async fn create_folder(parent: &str, name: &str) -> Result<(), String> {
    let parent = existing_directory(parent)?;
    let target = parent.join(valid_child_name(name)?);
    if target.exists() {
        return Err("A file or folder with that name already exists".to_string());
    }
    fs::create_dir(target)
        .await
        .map_err(|e| format!("Failed to create folder: {e}"))
}

pub async fn create_file(parent: &str, name: &str) -> Result<String, String> {
    let parent = existing_directory(parent)?;
    let target = parent.join(valid_child_name(name)?);
    if target.exists() {
        return Err("A file or folder with that name already exists".to_string());
    }
    fs::write(&target, "")
        .await
        .map_err(|e| format!("Failed to create file: {e}"))?;
    Ok(target.to_string_lossy().to_string())
}

pub async fn delete_path(path: &str) -> Result<(), String> {
    let original = Path::new(path);
    let root = crate::paths::canonical_xampp_root()?;
    let metadata = fs::symlink_metadata(original)
        .await
        .map_err(|e| format!("Failed to inspect item: {e}"))?;
    if metadata.file_type().is_symlink() {
        return Err("Symbolic links cannot be deleted from this app".to_string());
    }
    let canonical = crate::paths::ensure_existing_path_in_xampp(original)?;
    if canonical == root {
        return Err("The XAMPP root cannot be deleted".to_string());
    }
    let canonical_for_recycle = canonical.clone();
    tokio::task::spawn_blocking(move || crate::recycle::recycle_path(&canonical_for_recycle))
        .await
        .map_err(|e| format!("Delete task failed: {e}"))?
}

pub async fn rename_path(path: &str, new_name: &str) -> Result<String, String> {
    let original = Path::new(path);
    let metadata = fs::symlink_metadata(original)
        .await
        .map_err(|e| format!("Failed to inspect item: {e}"))?;
    if metadata.file_type().is_symlink() {
        return Err("Symbolic links cannot be renamed from this app".to_string());
    }
    let canonical = crate::paths::ensure_existing_path_in_xampp(original)?;
    if canonical == crate::paths::canonical_xampp_root()? {
        return Err("The XAMPP root cannot be renamed".to_string());
    }
    let parent = canonical
        .parent()
        .ok_or_else(|| "Item has no parent folder".to_string())?;
    let target = parent.join(valid_child_name(new_name)?);
    if target.exists() {
        return Err("A file or folder with that name already exists".to_string());
    }
    fs::rename(&canonical, &target)
        .await
        .map_err(|e| format!("Failed to rename item: {e}"))?;
    Ok(target.to_string_lossy().to_string())
}

pub async fn upload_files(destination: &str, source_paths: Vec<String>) -> Result<(), String> {
    if source_paths.is_empty() {
        return Err("Select one or more files to upload".to_string());
    }
    let destination = existing_directory(destination)?;
    let sources = source_paths
        .into_iter()
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    tokio::task::spawn_blocking(move || {
        let mut total_bytes = 0_u64;
        let mut planned = Vec::new();
        for source in sources {
            let metadata = std::fs::symlink_metadata(&source)
                .map_err(|e| format!("Failed to read upload: {e}"))?;
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return Err("Only normal files can be uploaded".to_string());
            }
            total_bytes += metadata.len();
            if total_bytes > 500 * 1024 * 1024 {
                return Err("Uploads are limited to 500 MB at a time".to_string());
            }
            let name = source
                .file_name()
                .ok_or_else(|| "Invalid upload filename".to_string())?;
            let target = destination.join(name);
            if target.exists() {
                return Err(format!(
                    "{} already exists in this folder",
                    name.to_string_lossy()
                ));
            }
            if planned
                .iter()
                .any(|(_, planned_target): &(PathBuf, PathBuf)| planned_target == &target)
            {
                return Err(format!(
                    "Multiple selected files are named {}",
                    name.to_string_lossy()
                ));
            }
            planned.push((source, target));
        }

        let mut copied = Vec::new();
        for (source, target) in planned {
            if let Err(error) = std::fs::copy(&source, &target) {
                for copied_path in copied {
                    let _ = std::fs::remove_file(copied_path);
                }
                return Err(format!("Failed to upload file: {error}"));
            }
            copied.push(target);
        }
        Ok(())
    })
    .await
    .map_err(|e| format!("Upload task failed: {e}"))?
}

pub async fn upload_folder(destination: &str, source_path: &str) -> Result<(), String> {
    let destination = existing_directory(destination)?;
    let source = std::fs::canonicalize(source_path)
        .map_err(|_| "The selected folder is no longer available".to_string())?;
    let source_metadata =
        std::fs::symlink_metadata(&source).map_err(|e| format!("Failed to inspect folder: {e}"))?;
    if source_metadata.file_type().is_symlink() || !source_metadata.is_dir() {
        return Err("Select a normal folder to upload".to_string());
    }
    let name = source
        .file_name()
        .ok_or_else(|| "Invalid folder name".to_string())?
        .to_owned();
    let target = destination.join(&name);
    if target.exists() {
        return Err("A folder with that name already exists".to_string());
    }
    if target.starts_with(&source) {
        return Err("Cannot upload a parent folder into itself".to_string());
    }
    tokio::task::spawn_blocking(move || copy_upload_tree(&source, &target))
        .await
        .map_err(|e| format!("Folder upload task failed: {e}"))?
}

fn copy_upload_tree(source: &Path, destination: &Path) -> Result<(), String> {
    const MAX_FILES: usize = 10_000;
    const MAX_BYTES: u64 = 500 * 1024 * 1024;
    fn copy_directory(
        source: &Path,
        destination: &Path,
        files: &mut usize,
        bytes: &mut u64,
    ) -> Result<(), String> {
        std::fs::create_dir_all(destination)
            .map_err(|e| format!("Failed to create folder: {e}"))?;
        for entry in std::fs::read_dir(source).map_err(|e| format!("Failed to read folder: {e}"))? {
            let entry = entry.map_err(|e| format!("Failed to read folder item: {e}"))?;
            let from = entry.path();
            let to = destination.join(entry.file_name());
            let kind = entry
                .file_type()
                .map_err(|e| format!("Failed to inspect folder item: {e}"))?;
            if kind.is_symlink() {
                return Err("Folder uploads cannot include symbolic links".to_string());
            }
            if kind.is_dir() {
                copy_directory(&from, &to, files, bytes)?;
            } else if kind.is_file() {
                *files += 1;
                *bytes += entry
                    .metadata()
                    .map_err(|e| format!("Failed to inspect file: {e}"))?
                    .len();
                if *files > MAX_FILES || *bytes > MAX_BYTES {
                    return Err("Folder upload is limited to 10,000 files or 500 MB".to_string());
                }
                std::fs::copy(from, to).map_err(|e| format!("Failed to upload file: {e}"))?;
            }
        }
        Ok(())
    }
    let mut files = 0;
    let mut bytes = 0;
    let result = copy_directory(source, destination, &mut files, &mut bytes);
    if result.is_err() {
        let _ = std::fs::remove_dir_all(destination);
    }
    result
}

/// Uploads a mixed list of dropped files and folders (from drag-and-drop) into
/// a destination folder, routing each item to the correct copy path.
pub async fn upload_paths(destination: &str, source_paths: Vec<String>) -> Result<(), String> {
    if source_paths.is_empty() {
        return Err("No files were dropped".to_string());
    }
    let destination = existing_directory(destination)?;
    let mut files = Vec::new();
    let mut folders = Vec::new();
    for source in &source_paths {
        let metadata = fs::symlink_metadata(source)
            .await
            .map_err(|e| format!("Failed to inspect dropped item: {e}"))?;
        if metadata.file_type().is_symlink() {
            return Err("Symbolic links cannot be uploaded".to_string());
        }
        if metadata.is_dir() {
            folders.push(source.clone());
        } else if metadata.is_file() {
            files.push(source.clone());
        } else {
            return Err(format!("Unsupported dropped item: {source}"));
        }
    }
    if !files.is_empty() {
        let destination = destination.to_string_lossy().to_string();
        upload_files(&destination, files).await?;
    }
    for folder in folders {
        let destination = destination.to_string_lossy().to_string();
        upload_folder(&destination, &folder).await?;
    }
    Ok(())
}

#[derive(serde::Serialize, Clone, Debug)]
pub struct ImageData {
    pub mime: String,
    pub data: String,
}

/// Reads an image file inside XAMPP and returns its MIME type plus base64
/// payload so the frontend can render a preview without extra plugins.
pub async fn read_image(path: &str) -> Result<ImageData, String> {
    use base64::Engine;

    const MAX_IMAGE_BYTES: u64 = 15 * 1024 * 1024;

    let canonical = crate::paths::ensure_existing_path_in_xampp(Path::new(path))?;
    if canonical.is_dir() {
        return Err("Cannot read a directory as an image".to_string());
    }
    let extension = canonical
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.to_ascii_lowercase());
    let mime = match extension.as_deref() {
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("bmp") => "image/bmp",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        _ => return Err("This file type cannot be previewed as an image".to_string()),
    };
    let metadata = fs::metadata(&canonical)
        .await
        .map_err(|e| format!("Failed to inspect image: {e}"))?;
    if metadata.len() > MAX_IMAGE_BYTES {
        return Err("This image is too large to preview (maximum 15 MB)".to_string());
    }
    let bytes = fs::read(&canonical)
        .await
        .map_err(|e| format!("Failed to read image: {e}"))?;
    Ok(ImageData {
        mime: mime.to_string(),
        data: base64::engine::general_purpose::STANDARD.encode(bytes),
    })
}
