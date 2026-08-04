use crate::database::QueryResult;

#[tauri::command]
pub async fn mysql_connect(
    host: Option<String>,
    port: Option<u16>,
    user: String,
    password: String,
) -> Result<(), crate::error::AppError> {
    crate::database::connect(host, port, user, password).await
}

#[tauri::command]
pub async fn mysql_disconnect() -> Result<(), crate::error::AppError> {
    crate::database::disconnect().await;
    Ok(())
}

#[tauri::command]
pub async fn list_databases() -> Result<Vec<String>, crate::error::AppError> {
    crate::database::list_databases().await
}

#[tauri::command]
pub async fn list_tables(database: String) -> Result<Vec<String>, crate::error::AppError> {
    crate::database::list_tables(&database).await
}

#[tauri::command]
pub async fn run_query(query: String) -> Result<QueryResult, crate::error::AppError> {
    crate::database::run_query(&query).await
}

#[tauri::command]
pub async fn export_database(database: String) -> Result<String, crate::error::AppError> {
    crate::database::export_database(&database).await
}
