use crate::ssl_manager::{CertFile, CertificateInfo};

#[tauri::command]
pub async fn list_certificates() -> Result<Vec<CertFile>, String> {
    crate::ssl_manager::list_certificates().await
}

#[tauri::command]
pub async fn read_certificate(path: String) -> Result<CertificateInfo, String> {
    crate::ssl_manager::read_certificate(&path).await
}

#[tauri::command]
pub async fn generate_self_signed(
    common_name: String,
    days: u32,
) -> Result<CertificateInfo, String> {
    crate::ssl_manager::generate_self_signed(common_name, days).await
}
