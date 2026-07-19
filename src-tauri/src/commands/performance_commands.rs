#[tauri::command]
pub async fn get_local_performance() -> Result<crate::performance::MachinePerformance, String> {
    crate::performance::get_local_performance().await
}
