use crate::file_browser::FileEntry;

#[tauri::command]
pub async fn list_directory(dir: String) -> Result<Vec<FileEntry>, String> {
    crate::file_browser::list_directory(&dir).await
}

#[tauri::command]
pub async fn read_file(path: String) -> Result<String, String> {
    crate::file_browser::read_file(&path).await
}

#[tauri::command]
pub async fn write_file(path: String, content: String) -> Result<(), String> {
    crate::file_browser::write_file(&path, &content).await
}

#[tauri::command]
pub async fn create_folder(parent: String, name: String) -> Result<(), String> {
    crate::file_browser::create_folder(&parent, &name).await
}
#[tauri::command]
pub async fn create_file(parent: String, name: String) -> Result<String, String> {
    crate::file_browser::create_file(&parent, &name).await
}
#[tauri::command]
pub async fn delete_path(path: String) -> Result<(), String> {
    crate::file_browser::delete_path(&path).await
}
#[tauri::command]
pub async fn rename_path(path: String, new_name: String) -> Result<String, String> {
    crate::file_browser::rename_path(&path, &new_name).await
}
#[tauri::command]
pub async fn upload_files(destination: String, source_paths: Vec<String>) -> Result<(), String> {
    crate::file_browser::upload_files(&destination, source_paths).await
}
#[tauri::command]
pub async fn upload_folder(destination: String, source_path: String) -> Result<(), String> {
    crate::file_browser::upload_folder(&destination, &source_path).await
}
