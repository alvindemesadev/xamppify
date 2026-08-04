#[tauri::command]
pub async fn search_htdocs(
    query: String,
    literal: bool,
) -> Result<Vec<crate::search::SearchMatch>, String> {
    crate::search::search_htdocs(query, literal).await
}
