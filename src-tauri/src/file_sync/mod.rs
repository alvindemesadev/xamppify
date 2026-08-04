pub mod history;

use serde::Serialize;
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct SyncResult {
    pub success: bool,
    pub output: String,
    pub files_copied: u64,
}

/// Establishes (and tears down) an authenticated SMB session to a remote host.
///
/// Uses the Win32 `WNetAddConnection2`/`WNetCancelConnection2` API instead of
/// `net use` so credentials never appear in a command line (visible in process
/// lists) and the session is always removed when dropped, even on early returns.
struct NetworkShareSession {
    host: String,
    connected: bool,
}

impl NetworkShareSession {
    async fn connect(host: &str, username: Option<&str>, password: Option<&str>) -> Self {
        let host = host.trim().to_string();
        let connected = if username.is_some_and(|user| !user.trim().is_empty()) {
            let host = host.clone();
            let username = username.unwrap_or("").trim().to_string();
            let password = password.unwrap_or("").to_string();
            tokio::task::spawn_blocking(move || wnet::connect(&host, &username, &password))
                .await
                .ok()
                .and_then(Result::ok)
                .is_some()
        } else {
            false
        };
        NetworkShareSession { host, connected }
    }

    fn disconnect(&self) {
        if self.connected {
            let _ = wnet::disconnect(&self.host);
        }
    }
}

impl Drop for NetworkShareSession {
    fn drop(&mut self) {
        self.disconnect();
    }
}

#[cfg(windows)]
mod wnet {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Foundation::ERROR_SUCCESS;
    use windows_sys::Win32::NetworkManagement::WNet::*;

    fn wide(value: &str) -> Vec<u16> {
        OsStr::new(value).encode_wide().chain(std::iter::once(0)).collect()
    }

    pub fn connect(host: &str, username: &str, password: &str) -> Result<(), String> {
        let remote_name = wide(host);
        let user = wide(username);
        let pass = wide(password);

        let resource = NETRESOURCEW {
            dwScope: 0,
            dwType: RESOURCETYPE_DISK,
            dwDisplayType: 0,
            dwUsage: RESOURCEUSAGE_CONNECTABLE,
            lpLocalName: std::ptr::null_mut(),
            lpRemoteName: remote_name.as_ptr() as *mut _,
            lpComment: std::ptr::null_mut(),
            lpProvider: std::ptr::null_mut(),
        };

        // CONNECT_TEMPORARY keeps the session out of the persistent credential store.
        let code = unsafe { WNetAddConnection2W(&resource, pass.as_ptr(), user.as_ptr(), CONNECT_TEMPORARY) };
        if code == ERROR_SUCCESS {
            Ok(())
        } else {
            Err(format!("SMB connection failed with Windows error code {code} (0x{code:08X})"))
        }
    }

    pub fn disconnect(host: &str) -> Result<(), String> {
        let remote_name = wide(host);
        let code = unsafe { WNetCancelConnection2W(remote_name.as_ptr(), 0, 1) };
        if code == ERROR_SUCCESS {
            Ok(())
        } else {
            Err(format!("SMB disconnection failed with Windows error code {code} (0x{code:08X})"))
        }
    }
}

#[cfg(not(windows))]
mod wnet {
    pub fn connect(_host: &str, _username: &str, _password: &str) -> Result<(), String> {
        Err("Network share credentials are only supported on Windows".to_string())
    }

    pub fn disconnect(_host: &str) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Serialize)]
pub struct ConnectionTestResult {
    pub ping_ok: bool,
    pub smb_port_ok: bool,
    pub share_accessible: bool,
    pub unc_path: String,
    pub message: String,
    pub suggestions: Vec<String>,
}

pub fn build_unc_path(remote_host: &str, destination: &str) -> String {
    let clean_host = remote_host.trim();
    let clean_dest = destination.trim().replace('/', "\\");

    if clean_dest.starts_with(r"\\") {
        clean_dest
    } else if clean_dest.len() >= 2 && clean_dest.as_bytes()[1] == b':' {
        let drive = &clean_dest[..1];
        let rest = clean_dest[2..].trim_start_matches('\\');
        format!(r"\\{}\{}$\{}", clean_host, drive, rest)
    } else {
        let rest = clean_dest.trim_start_matches('\\');
        format!(r"\\{}\{}", clean_host, rest)
    }
}

pub async fn sync_to_remote(
    source: &str,
    destination: &str,
    remote_host: &str,
    username: Option<&str>,
    password: Option<&str>,
) -> Result<SyncResult, crate::error::AppError> {
    let source_path = Path::new(source);
    if !source_path.exists() {
        return Err(format!("Source path does not exist: {}", source).into());
    }

    let dest_unc = build_unc_path(remote_host, destination);

    // Establish an optional authenticated SMB session. The session is dropped
    // (disconnected) when `session` goes out of scope, so credentials never
    // linger in the persistent connection cache.
    let session = NetworkShareSession::connect(remote_host, username, password).await;

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
        // Count every affected file/dir line: new, updated, older-overwritten,
        // tweaked, and extras. "Same", "Failed", and the (suppressed) summary
        // lines are excluded because those rows were not copied.
        const COPIED_STATUSES: [&str; 6] =
            ["New File", "New Dir", "Newer", "Changed", "Tweaked", "Extra"];
        stdout
            .lines()
            .filter(|line| COPIED_STATUSES.iter().any(|status| line.contains(status)))
            .count() as u64
    } else {
        0
    };

    drop(session);

    let mut final_output = stdout;
    if !success {
        if final_output.contains("ERROR 53") || final_output.contains("0x00000035") || final_output.contains("network path was not found") {
            final_output.push_str(&format!(
                "\n===================================================================\n\
                 DIAGNOSTIC HINT: ERROR 53 (0x00000035) - Network Path Not Found\n\
                 -------------------------------------------------------------------\n\
                 Windows could not reach UNC share '{dest_unc}'.\n\n\
                 Troubleshooting Checklist:\n\
                 1. PING: Ensure host '{remote_host}' is powered on & reachable.\n\
                 2. FIREWALL: Allow 'File and Printer Sharing (SMB-In)' on remote host.\n\
                 3. NETWORK PROFILE: Set remote network profile to 'Private'.\n\
                 4. ADMIN SHARE (C$): Run this command in Admin PowerShell on target host:\n\
                    reg add \"HKLM\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Policies\\System\" /v LocalAccountTokenFilterPolicy /t REG_DWORD /d 1 /f\n\
                 5. CREDENTIALS: Fill in Remote Username & Password if auth is required.\n\
                 ===================================================================\n"
            ));
        } else if final_output.contains("ERROR 5 ") || final_output.contains("0x00000005") || final_output.contains("Access is denied") {
            final_output.push_str(&format!(
                "\n===================================================================\n\
                 DIAGNOSTIC HINT: ERROR 5 (0x00000005) - Access Denied\n\
                 -------------------------------------------------------------------\n\
                 Permission denied accessing '{dest_unc}'.\n\n\
                 Troubleshooting Checklist:\n\
                 1. Fill in valid remote Administrator Username & Password.\n\
                 2. Ensure your remote account has read/write permissions to destination.\n\
                 ===================================================================\n"
            ));
        } else if final_output.contains("ERROR 67") || final_output.contains("0x00000043") {
            final_output.push_str(&format!(
                "\n===================================================================\n\
                 DIAGNOSTIC HINT: ERROR 67 (0x00000043) - Network Name Not Found\n\
                 -------------------------------------------------------------------\n\
                 The share name in '{dest_unc}' does not exist on '{remote_host}'.\n\
                 Check the folder name or administrative share (e.g. C$).\n\
                 ===================================================================\n"
            ));
        }
    }

    Ok(SyncResult {
        success,
        output: final_output,
        files_copied,
    })
}

pub async fn test_remote_connection(
    remote_host: &str,
    destination: &str,
    username: Option<&str>,
    password: Option<&str>,
) -> ConnectionTestResult {
    let unc_path = build_unc_path(remote_host, destination);
    let mut suggestions = Vec::new();

    // 1. Check Ping
    let mut ping_cmd = tokio::process::Command::new("ping");
    ping_cmd.args(["-n", "1", "-w", "1500", remote_host.trim()]);
    crate::paths::hide_console_for_tokio_command(&mut ping_cmd);
    let ping_ok = match ping_cmd.output().await {
        Ok(out) => out.status.success(),
        Err(_) => false,
    };

    if !ping_ok {
        suggestions.push(format!(
            "Host '{}' is not responding to ping. Check if target computer is powered on and connected to the network.",
            remote_host
        ));
    }

    // 2. Check SMB Port 445
    let host_addr = format!("{}:445", remote_host.trim());
    let smb_port_ok = matches!(
        tokio::time::timeout(
            std::time::Duration::from_millis(1500),
            tokio::net::TcpStream::connect(&host_addr),
        )
        .await,
        Ok(Ok(_))
    );

    if !smb_port_ok {
        suggestions.push(format!(
            "Port 445 (SMB) on '{}' is closed or blocked. Ensure Windows Firewall allows 'File and Printer Sharing' and Network Profile is set to Private.",
            remote_host
        ));
    }

    // 3. Optional auth (session is auto-disconnected when dropped)
    let session = NetworkShareSession::connect(remote_host, username, password).await;

    // 4. Check Share Access
    let path_obj = Path::new(&unc_path);
    let share_accessible = path_obj.exists();

    drop(session);

    if !share_accessible && smb_port_ok {
        suggestions.push(format!(
            "SMB port 445 is open, but share '{}' is inaccessible. If targeting C$, run 'reg add \"HKLM\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Policies\\System\" /v LocalAccountTokenFilterPolicy /t REG_DWORD /d 1 /f' on target host in Admin PowerShell.",
            unc_path
        ));
    }

    let message = if ping_ok && smb_port_ok && share_accessible {
        format!("Successfully reached UNC share {}", unc_path)
    } else if !ping_ok {
        format!("Host {} is unreachable (Ping failed)", remote_host)
    } else if !smb_port_ok {
        format!("Host {} reached, but SMB port 445 is blocked", remote_host)
    } else {
        format!("Host {} reached, but share {} is inaccessible", remote_host, unc_path)
    };

    ConnectionTestResult {
        ping_ok,
        smb_port_ok,
        share_accessible,
        unc_path,
        message,
        suggestions,
    }
}

#[cfg(test)]
mod tests {
    use super::build_unc_path;

    #[test]
    fn passes_through_full_unc_paths() {
        assert_eq!(
            build_unc_path("pc-01", r"\\pc-01\C$\www"),
            r"\\pc-01\C$\www"
        );
    }

    #[test]
    fn converts_drive_paths_to_admin_shares() {
        assert_eq!(
            build_unc_path("192.168.1.5", r"C:\www\site"),
            r"\\192.168.1.5\C$\www\site"
        );
    }

    #[test]
    fn normalizes_forward_slashes_in_drive_paths() {
        assert_eq!(
            build_unc_path("pc-01", "C:/www/site"),
            r"\\pc-01\C$\www\site"
        );
    }

    #[test]
    fn treats_plain_paths_as_share_relative() {
        assert_eq!(
            build_unc_path("pc-01", "shared/folder"),
            r"\\pc-01\shared\folder"
        );
    }

    #[test]
    fn trims_surrounding_whitespace() {
        assert_eq!(
            build_unc_path("  pc-01  ", "  C:\\www  "),
            r"\\pc-01\C$\www"
        );
    }
}
