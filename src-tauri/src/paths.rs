use std::path::{Path, PathBuf};

pub fn xampp_root() -> PathBuf {
    std::env::var_os("XAMPP_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\xampp"))
}

pub fn apache_log_path() -> PathBuf {
    xampp_root().join("apache").join("logs").join("error.log")
}

pub fn mysql_log_path() -> PathBuf {
    xampp_root().join("mysql").join("data").join("mysql.err")
}

/// Finds the MySQL/MariaDB error log created by the installed XAMPP version.
///
/// XAMPP installations do not consistently use `mysql.err`; MariaDB may use
/// `mysql_error.log`, `<hostname>.err`, or another `.err` file in `data`.
/// Keep the search limited to the XAMPP MySQL directories.
pub fn find_mysql_log_path() -> Option<PathBuf> {
    let mysql_root = xampp_root().join("mysql");
    let data_dir = mysql_root.join("data");
    let preferred = [
        mysql_log_path(),
        data_dir.join("mysql_error.log"),
        data_dir.join("mariadb.err"),
        mysql_root.join("mysql.err"),
        mysql_root.join("mysql_error.log"),
    ];

    if let Some(path) = preferred.into_iter().find(|path| path.is_file()) {
        return Some(path);
    }

    for directory in [data_dir, mysql_root] {
        let Ok(entries) = std::fs::read_dir(directory) else {
            continue;
        };
        if let Some(path) = entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .find(|path| {
                path.is_file()
                    && path.extension().is_some_and(|extension| {
                        extension.eq_ignore_ascii_case("err")
                            || extension.eq_ignore_ascii_case("log")
                    })
            })
        {
            return Some(path);
        }
    }

    None
}

pub fn ssl_crt_dir() -> PathBuf {
    xampp_root().join("apache").join("conf").join("ssl.crt")
}

pub fn ssl_key_dir() -> PathBuf {
    xampp_root().join("apache").join("conf").join("ssl.key")
}

pub fn openssl_path() -> PathBuf {
    xampp_root().join("apache").join("bin").join("openssl.exe")
}

/// Removes Windows' extended-length path prefix before passing a path to
/// third-party programs such as the OpenSSL bundled with XAMPP. Some OpenSSL
/// builds treat `\\?\C:\...` as an invalid relative path.
pub fn path_for_external_command(path: &Path) -> PathBuf {
    #[cfg(windows)]
    {
        if let Some(path) = path.to_string_lossy().strip_prefix(r"\\?\") {
            return PathBuf::from(path);
        }
    }
    path.to_path_buf()
}

#[cfg(windows)]
pub fn hide_console_for_tokio_command(command: &mut tokio::process::Command) {
    // CREATE_NO_WINDOW prevents short-lived cmd windows from flashing when a
    // GUI application starts OpenSSL or other console executables.
    command.creation_flags(0x0800_0000);
}

#[cfg(not(windows))]
pub fn hide_console_for_tokio_command(_command: &mut tokio::process::Command) {}

#[cfg(windows)]
pub fn hide_console_for_std_command(command: &mut std::process::Command) {
    use std::os::windows::process::CommandExt;
    command.creation_flags(0x0800_0000);
}

#[cfg(not(windows))]
pub fn hide_console_for_std_command(_command: &mut std::process::Command) {}

pub fn canonical_xampp_root() -> Result<PathBuf, String> {
    std::fs::canonicalize(xampp_root()).map_err(|e| format!("XAMPP root is unavailable: {e}"))
}

pub fn ensure_existing_path_in_xampp(path: &Path) -> Result<PathBuf, String> {
    let root = canonical_xampp_root()?;
    let canonical =
        std::fs::canonicalize(path).map_err(|e| format!("Unable to resolve path: {e}"))?;

    if canonical.starts_with(&root) {
        Ok(canonical)
    } else {
        Err("Path must be inside the configured XAMPP directory".to_string())
    }
}

pub fn ensure_writable_path_in_xampp(path: &Path) -> Result<PathBuf, String> {
    let root = canonical_xampp_root()?;
    let parent = path
        .parent()
        .ok_or_else(|| "Path has no parent directory".to_string())?;
    let canonical_parent = std::fs::canonicalize(parent)
        .map_err(|e| format!("Unable to resolve parent directory: {e}"))?;

    if !canonical_parent.starts_with(&root) {
        return Err("Path must be inside the configured XAMPP directory".to_string());
    }

    let file_name = path
        .file_name()
        .ok_or_else(|| "Path has no file name".to_string())?;
    let candidate = canonical_parent.join(file_name);
    if candidate.exists() {
        return ensure_existing_path_in_xampp(&candidate);
    }

    Ok(candidate)
}
