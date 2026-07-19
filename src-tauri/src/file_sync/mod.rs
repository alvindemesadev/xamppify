use serde::Serialize;
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct SyncResult {
    pub success: bool,
    pub output: String,
    pub files_copied: u64,
}

pub async fn sync_to_remote(
    source: &str,
    destination: &str,
    remote_host: &str,
) -> Result<SyncResult, String> {
    let source_path = Path::new(source);
    if !source_path.exists() {
        return Err(format!("Source path does not exist: {}", source));
    }

    let dest_unc = format!(r"\\{}\{}", remote_host, destination.replace(':', "$"));

    let mut command = tokio::process::Command::new("robocopy");
    command
        .arg(source)
        .arg(&dest_unc)
        .arg("/MIR")
        .arg("/R:2")
        .arg("/W:2")
        .arg("/NP")
        .arg("/NJH")
        .arg("/NJS")
        .arg("/NDL");

    crate::paths::hide_console_for_tokio_command(&mut command);

    let output = command
        .output()
        .await
        .map_err(|e| format!("Failed to run robocopy: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let exit_code = output.status.code().unwrap_or(-1);

    // robocopy exit codes: 0-7 = success, 8+ = error
    let success = exit_code < 8;
    let files_copied = if success {
        stdout
            .lines()
            .filter(|l| l.contains("New File") || l.contains("New Dir"))
            .count() as u64
    } else {
        0
    };

    Ok(SyncResult {
        success,
        output: stdout,
        files_copied,
    })
}
