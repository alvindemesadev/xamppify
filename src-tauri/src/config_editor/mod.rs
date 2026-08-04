use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ConfigFile {
    pub name: String,
    pub path: String,
    pub category: String,
}

pub fn known_configs() -> Vec<ConfigFile> {
    let root = crate::paths::xampp_root();
    let configs = vec![
        ConfigFile {
            name: "httpd.conf".into(),
            path: root
                .join("apache\\conf\\httpd.conf")
                .to_string_lossy()
                .to_string(),
            category: "Apache".into(),
        },
        ConfigFile {
            name: "php.ini".into(),
            path: root.join("php\\php.ini").to_string_lossy().to_string(),
            category: "PHP".into(),
        },
        ConfigFile {
            name: "my.ini".into(),
            path: root
                .join("mysql\\bin\\my.ini")
                .to_string_lossy()
                .to_string(),
            category: "MySQL".into(),
        },
        ConfigFile {
            name: "httpd-vhosts.conf".into(),
            path: root
                .join("apache\\conf\\extra\\httpd-vhosts.conf")
                .to_string_lossy()
                .to_string(),
            category: "Apache".into(),
        },
        ConfigFile {
            name: "httpd-ssl.conf".into(),
            path: root
                .join("apache\\conf\\extra\\httpd-ssl.conf")
                .to_string_lossy()
                .to_string(),
            category: "Apache".into(),
        },
        ConfigFile {
            name: "phpmyadmin.conf".into(),
            path: root
                .join("phpMyAdmin\\config.inc.php")
                .to_string_lossy()
                .to_string(),
            category: "phpMyAdmin".into(),
        },
    ];
    configs
        .into_iter()
        .filter(|config| std::path::Path::new(&config.path).is_file())
        .collect()
}

#[derive(Debug, Serialize)]
pub struct IniSection {
    pub name: String,
    pub line: usize,
}

pub fn parse_ini_sections(content: &str) -> Vec<IniSection> {
    content
        .lines()
        .enumerate()
        .filter(|(_, line)| line.trim().starts_with('[') && line.trim().ends_with(']'))
        .map(|(i, line)| IniSection {
            name: line.trim().trim_matches('[').trim_matches(']').to_string(),
            line: i,
        })
        .collect()
}

#[derive(Debug, Serialize)]
pub struct ConfigTestResult {
    pub ok: bool,
    pub output: String,
}

/// Runs the Apache bundled with XAMPP in configuration-test mode (`httpd -t`).
/// Reports the exit status and the combined output so the caller can show the
/// exact configuration error, not just a generic "failed".
pub async fn test_apache_config() -> Result<ConfigTestResult, String> {
    let apache_bin = crate::paths::xampp_root().join(r"apache\bin");
    let httpd = apache_bin.join("httpd.exe");
    if !httpd.is_file() {
        return Err("Apache executable not found; XAMPP may not be installed here".to_string());
    }

    let output = tokio::process::Command::new(&httpd)
        .arg("-t")
        .current_dir(&apache_bin)
        .output()
        .await
        .map_err(|e| format!("Failed to run httpd -t: {e}"))?;

    let mut text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if !stderr.is_empty() {
        if !text.is_empty() {
            text.push('\n');
        }
        text.push_str(&stderr);
    }

    Ok(ConfigTestResult {
        ok: output.status.success(),
        output: text,
    })
}

/// Saves a known XAMPP configuration file after snapshotting the current
/// contents into `<app_data>/config-backups/`. Returns the backup path.
pub async fn save_config_file(path: &std::path::Path, content: &str, app_data_dir: &std::path::Path) -> Result<String, String> {
    let canonical = crate::paths::ensure_existing_path_in_xampp(path)?;
    let name = canonical
        .file_name()
        .ok_or_else(|| "Path has no file name".to_string())?
        .to_string_lossy()
        .to_string();
    let backups_dir = app_data_dir.join("config-backups");
    tokio::fs::create_dir_all(&backups_dir)
        .await
        .map_err(|e| format!("Failed to create config backups folder: {e}"))?;
    let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
    let backup_path = backups_dir.join(format!("{name}-{timestamp}"));
    tokio::fs::copy(&canonical, &backup_path)
        .await
        .map_err(|e| format!("Failed to back up the current configuration: {e}"))?;
    tokio::fs::write(&canonical, content)
        .await
        .map_err(|e| format!("Failed to write configuration file: {e}"))?;
    Ok(backup_path.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::parse_ini_sections;

    #[test]
    fn finds_all_sections_with_zero_based_lines() {
        let content = "; comment\n[mysql]\nport=3306\n\n[mysqld]\nmax_connections=100\n";
        let sections = parse_ini_sections(content);
        assert_eq!(sections.len(), 2);
        assert_eq!(sections[0].name, "mysql");
        assert_eq!(sections[0].line, 1);
        assert_eq!(sections[1].name, "mysqld");
        assert_eq!(sections[1].line, 4);
    }

    #[test]
    fn ignores_brackets_inside_values() {
        let content = "[section]\nkey = [not a section]\n";
        let sections = parse_ini_sections(content);
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].name, "section");
    }

    #[test]
    fn returns_empty_for_no_sections() {
        assert!(parse_ini_sections("a=1\nb=2\n").is_empty());
        assert!(parse_ini_sections("").is_empty());
    }
}
