use serde::Serialize;
use std::net::IpAddr;
use std::path::Path;
use tokio::fs;

#[derive(Debug, Serialize)]
pub struct CertificateInfo {
    pub name: String,
    pub cert_path: String,
    pub key_path: Option<String>,
    pub subject: String,
    pub issuer: String,
    pub valid_from: String,
    pub valid_to: String,
    pub san: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct CertFile {
    pub name: String,
    pub path: String,
    pub is_key: bool,
}

pub async fn list_certificates() -> Result<Vec<CertFile>, String> {
    let mut certs = Vec::new();
    collect_cert_files(&crate::paths::ssl_crt_dir(), false, &mut certs).await?;
    collect_cert_files(&crate::paths::ssl_key_dir(), true, &mut certs).await?;
    certs.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(certs)
}

async fn collect_cert_files(
    dir: &Path,
    is_key: bool,
    certs: &mut Vec<CertFile>,
) -> Result<(), String> {
    let mut entries = match fs::read_dir(dir).await {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(format!("Failed to read certificate directory: {e}")),
    };

    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|e| format!("Failed to read certificate entry: {e}"))?
    {
        let path = entry.path();
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or_default();
        let allowed = if is_key {
            matches!(extension, "key" | "pem")
        } else {
            matches!(extension, "crt" | "cer" | "pem")
        };
        if allowed {
            certs.push(CertFile {
                name: entry.file_name().to_string_lossy().to_string(),
                path: path.to_string_lossy().to_string(),
                is_key,
            });
        }
    }
    Ok(())
}

pub async fn read_certificate(path: &str) -> Result<CertificateInfo, String> {
    let path = crate::paths::ensure_existing_path_in_xampp(Path::new(path))?;
    let cert_dir = crate::paths::ensure_existing_path_in_xampp(&crate::paths::ssl_crt_dir())?;
    if !path.starts_with(&cert_dir) {
        return Err(
            "Only certificates in the XAMPP SSL certificate directory can be inspected".to_string(),
        );
    }

    let openssl = crate::paths::openssl_path();
    if !openssl.exists() {
        return Err(format!("OpenSSL binary not found at {}", openssl.display()));
    }

    if !path.is_file() {
        return Err("The selected certificate file is no longer available".to_string());
    }
    let mut command =
        tokio::process::Command::new(crate::paths::path_for_external_command(&openssl));
    crate::paths::hide_console_for_tokio_command(&mut command);
    let output = command
        .args(["x509", "-in"])
        .arg(crate::paths::path_for_external_command(&path))
        .args([
            "-noout",
            "-subject",
            "-issuer",
            "-dates",
            "-ext",
            "subjectAltName",
        ])
        .output()
        .await
        .map_err(|e| format!("Failed to run OpenSSL: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "OpenSSL could not read certificate: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    let details = String::from_utf8_lossy(&output.stdout);
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let key_path = matching_key_path(&path);

    Ok(CertificateInfo {
        name,
        cert_path: path.to_string_lossy().to_string(),
        key_path,
        subject: prefixed_value(&details, "subject=").unwrap_or_default(),
        issuer: prefixed_value(&details, "issuer=").unwrap_or_default(),
        valid_from: prefixed_value(&details, "notBefore=").unwrap_or_default(),
        valid_to: prefixed_value(&details, "notAfter=").unwrap_or_default(),
        san: parse_sans(&details),
    })
}

pub async fn generate_self_signed(
    common_name: String,
    days: u32,
) -> Result<CertificateInfo, String> {
    let common_name = common_name.trim();
    if common_name.is_empty()
        || common_name.len() > 253
        || common_name.chars().any(char::is_control)
    {
        return Err("Common name must be between 1 and 253 printable characters".to_string());
    }
    if !(1..=3650).contains(&days) {
        return Err("Certificate lifetime must be between 1 and 3650 days".to_string());
    }

    let openssl = crate::paths::openssl_path();
    if !openssl.exists() {
        return Err(format!("OpenSSL binary not found at {}", openssl.display()));
    }

    let filename = safe_filename(common_name);
    let key_path = crate::paths::ssl_key_dir().join(format!("{filename}.key"));
    let cert_path = crate::paths::ssl_crt_dir().join(format!("{filename}.crt"));
    if key_path.exists() || cert_path.exists() {
        return Err(format!(
            "A certificate named '{filename}' already exists; refusing to overwrite it"
        ));
    }
    fs::create_dir_all(crate::paths::ssl_key_dir())
        .await
        .map_err(|e| format!("Failed to create SSL key directory: {e}"))?;
    fs::create_dir_all(crate::paths::ssl_crt_dir())
        .await
        .map_err(|e| format!("Failed to create SSL certificate directory: {e}"))?;

    let san = if common_name.parse::<IpAddr>().is_ok() {
        format!("subjectAltName=IP:{common_name}")
    } else {
        format!("subjectAltName=DNS:{common_name}")
    };
    let days = days.to_string();
    let mut command =
        tokio::process::Command::new(crate::paths::path_for_external_command(&openssl));
    crate::paths::hide_console_for_tokio_command(&mut command);
    let output = command
        .args([
            "req", "-x509", "-nodes", "-days", &days, "-newkey", "rsa:2048",
        ])
        .args(["-keyout"])
        .arg(crate::paths::path_for_external_command(&key_path))
        .args(["-out"])
        .arg(crate::paths::path_for_external_command(&cert_path))
        .args(["-subj", &format!("/CN={common_name}"), "-addext", &san])
        .output()
        .await
        .map_err(|e| format!("Failed to run OpenSSL: {e}"))?;
    if !output.status.success() {
        let _ = fs::remove_file(&key_path).await;
        let _ = fs::remove_file(&cert_path).await;
        return Err(format!(
            "OpenSSL failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    read_certificate(&cert_path.to_string_lossy()).await
}

fn matching_key_path(cert_path: &Path) -> Option<String> {
    let stem = cert_path.file_stem()?.to_string_lossy();
    let key = crate::paths::ssl_key_dir().join(format!("{stem}.key"));
    key.exists().then(|| key.to_string_lossy().to_string())
}

fn prefixed_value(content: &str, prefix: &str) -> Option<String> {
    content.lines().find_map(|line| {
        line.trim()
            .strip_prefix(prefix)
            .map(|value| value.trim().to_string())
    })
}

fn parse_sans(content: &str) -> Vec<String> {
    content
        .lines()
        .flat_map(|line| line.split(',').collect::<Vec<_>>())
        .filter_map(|value| {
            let value = value.trim();
            value
                .strip_prefix("DNS:")
                .or_else(|| value.strip_prefix("IP Address:"))
                .map(|value| value.trim().to_string())
        })
        .collect()
}

fn safe_filename(common_name: &str) -> String {
    let filename: String = common_name
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_') {
                ch
            } else {
                '_'
            }
        })
        .collect();
    filename
        .trim_matches('_')
        .to_string()
        .chars()
        .take(96)
        .collect::<String>()
        .max("certificate".to_string())
}

#[cfg(test)]
mod tests {
    use super::{parse_sans, safe_filename};

    #[test]
    fn sanitizes_certificate_filename() {
        assert_eq!(safe_filename("dev site/example"), "dev_site_example");
    }

    #[test]
    fn parses_dns_and_ip_sans() {
        let details = "X509v3 Subject Alternative Name:\n    DNS:localhost, DNS:example.test, IP Address:127.0.0.1\n";
        assert_eq!(
            parse_sans(details),
            ["localhost", "example.test", "127.0.0.1"]
        );
    }
}
