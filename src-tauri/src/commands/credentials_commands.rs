#[tauri::command]
pub fn save_mysql_credentials(user: String, password: String) -> Result<(), String> {
    crate::credentials::save_mysql_credentials(&user, &password)
}

#[tauri::command]
pub fn get_mysql_credentials() -> Result<Option<crate::credentials::MysqlCredentials>, String> {
    crate::credentials::get_mysql_credentials()
}

#[tauri::command]
pub fn delete_mysql_credentials() -> Result<(), String> {
    crate::credentials::delete_mysql_credentials()
}
