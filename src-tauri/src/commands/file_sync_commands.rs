#[tauri::command]
pub async fn sync_to_remote(
    source: String,
    destination: String,
    remote_host: String,
) -> Result<crate::file_sync::SyncResult, String> {
    crate::file_sync::sync_to_remote(&source, &destination, &remote_host).await
}
