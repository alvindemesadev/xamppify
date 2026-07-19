use crate::LogLine;

pub async fn read_apache_log(lines: Option<usize>) -> Result<Vec<LogLine>, String> {
    let content = tokio::fs::read_to_string(crate::paths::apache_log_path())
        .await
        .map_err(|e| format!("Failed to read Apache log: {}", e))?;

    let parsed = crate::log::parser::parse_apache_log(&content, lines);
    Ok(parsed)
}

pub async fn read_mysql_log(lines: Option<usize>) -> Result<Vec<LogLine>, String> {
    let log_path = crate::paths::find_mysql_log_path().ok_or_else(|| {
        "MySQL has not created an error log in this XAMPP installation yet. Start MySQL, then refresh this page."
            .to_string()
    })?;
    let content = tokio::fs::read_to_string(&log_path)
        .await
        .map_err(|_| "The MySQL error log could not be read. Check that XAMPP is running and that its log file is accessible.".to_string())?;

    let parsed = crate::log::parser::parse_mysql_log(&content, lines);
    Ok(parsed)
}
