use std::path::PathBuf;

#[derive(Debug, Clone, serde::Serialize)]
pub struct VhostEntry {
    pub domain: String,
    pub root: String,
    pub port: u16,
    pub ssl: bool,
}

pub fn vhosts_conf_path() -> PathBuf {
    crate::paths::vhosts_conf_path()
}

fn sanitize_domain(domain: &str) -> Result<String, String> {
    let domain = domain.trim().trim_start_matches("http://").trim_start_matches("https://");
    let domain = domain.trim_end_matches('/');
    if domain.is_empty() || domain.len() > 253 || domain.contains([' ', '\t', '\n']) {
        return Err("Enter a valid domain such as myapp.test".to_string());
    }
    if domain.chars().any(|c| !(c.is_ascii_alphanumeric() || matches!(c, '-' | '.' | ':'))) {
        return Err("Domains may contain letters, numbers, hyphens, dots, and colons".to_string());
    }
    Ok(domain.to_lowercase())
}

/// Parses existing VirtualHost blocks from httpd-vhosts.conf.
pub fn list_vhosts() -> Vec<VhostEntry> {
    let path = vhosts_conf_path();
    let Ok(content) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    let mut entries = Vec::new();
    let mut in_block = false;
    let mut domain = String::new();
    let mut root = String::new();
    let mut port = 80;
    let mut ssl = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("<VirtualHost") {
            in_block = true;
            domain.clear();
            root.clear();
            port = 80;
            ssl = false;
            // Extract port from "<VirtualHost *:8080>" or ":443"
            if let Some(port_str) = trimmed
                .trim_start_matches("<VirtualHost ")
                .split('>')
                .next()
                .and_then(|bind| bind.rsplit(':').next())
                .map(|p| p.trim_matches(['*', ' ', ':', ']', '[']))
            {
                if let Ok(p) = port_str.parse::<u16>() {
                    port = p;
                }
            }
            if port == 443 {
                ssl = true;
            }
        } else if in_block {
            if let Some(server) = trimmed.strip_prefix("ServerName ") {
                domain = server.trim().to_string();
            }
            if let Some(doc) = trimmed.strip_prefix("DocumentRoot ") {
                root = doc.trim().trim_matches('"').to_string();
            }
            if trimmed == "</VirtualHost>" {
                if !domain.is_empty() && !root.is_empty() {
                    entries.push(VhostEntry {
                        domain: domain.clone(),
                        root: root.clone(),
                        port,
                        ssl,
                    });
                }
                in_block = false;
            }
        }
    }
    entries
}

pub fn vhost_for_deployment(name: &str) -> Option<VhostEntry> {
    list_vhosts().into_iter().find(|entry| {
        let root = entry.root.replace('/', "\\");
        let expected = format!(r"{}\htdocs\{name}", crate::paths::xampp_root().to_string_lossy());
        entry.domain == format!("{name}.test")
            || root.trim_end_matches(['/', '\\']).eq_ignore_ascii_case(expected.trim_end_matches(['/', '\\']))
            || root.contains(name)
    })
}

/// Adds or removes a VirtualHost entry for a deployment. When `domain` is empty,
/// removes any existing entry for that deployment name.
pub fn set_vhost(name: &str, domain: Option<&str>, ssl: bool) -> Result<VhostEntry, String> {
    let path = vhosts_conf_path();
    if !path.exists() {
        return Err("httpd-vhosts.conf not found; XAMPP may not be installed here".to_string());
    }
    let mut content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read httpd-vhosts.conf: {e}"))?;

    // Remove any existing blocks whose DocumentRoot points at this deployment.
    let block_marker = format!(r"htdocs\{name}");
    let lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
    let mut output: Vec<String> = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let line = &lines[i];
        if line.trim_start().starts_with("<VirtualHost") {
            // Find matching closing tag
            let mut j = i;
            while j < lines.len() && lines[j].trim() != "</VirtualHost>" {
                j += 1;
            }
            let block = lines[i..=j].join("\n");
            if block.contains(&block_marker) {
                i = j + 1;
                continue;
            }
        }
        output.push(line.clone());
        i += 1;
    }
    content = output.join("\n");

    let domain = match domain {
        Some(d) => sanitize_domain(d)?,
        None => format!("{name}.test"),
    };
    let port = crate::paths::apache_port();
    let root = crate::paths::xampp_root()
        .join("htdocs")
        .join(name)
        .to_string_lossy()
        .to_string();
    let root_forward = root.replace('\\', "/");
    let listen_ssl = if ssl {
        format!(
            "\n<IfModule ssl_module>\n<VirtualHost *:443>\n    ServerName {domain}\n    DocumentRoot \"{root_forward}\"\n    SSLEngine on\n    SSLCertificateFile \"C:/xampp/apache/conf/ssl.crt/{name}.crt\"\n    SSLCertificateKeyFile \"C:/xampp/apache/conf/ssl.key/{name}.key\"\n    <Directory \"{root_forward}\">\n        Options Indexes FollowSymLinks\n        AllowOverride All\n        Require all granted\n    </Directory>\n</VirtualHost>\n</IfModule>\n"
        )
    } else {
        String::new()
    };
    let block = format!(
        "\n<VirtualHost *:{port}>\n    ServerName {domain}\n    DocumentRoot \"{root_forward}\"\n    <Directory \"{root_forward}\">\n        Options Indexes FollowSymLinks\n        AllowOverride All\n        Require all granted\n    </Directory>\n</VirtualHost>{listen_ssl}\n"
    );

    if !content.trim_end().ends_with('\n') {
        content.push('\n');
    }
    content.push_str(&block);

    std::fs::write(&path, content)
        .map_err(|e| format!("Failed to write httpd-vhosts.conf: {e}"))?;

    Ok(VhostEntry {
        domain,
        root: root_forward,
        port: if ssl { 443 } else { port },
        ssl,
    })
}

/// Adds an entry to the Windows hosts file (or removes it when domain is None).
/// Requires the app to run with write access; failures are surfaced to the UI.
pub fn set_hosts_entry(domain: Option<&str>) -> Result<(), String> {
    let path = crate::paths::hosts_file_path();
    let Ok(mut content) = std::fs::read_to_string(&path) else {
        return Err("Could not read the Windows hosts file".to_string());
    };
    if let Some(domain) = domain {
        let domain = sanitize_domain(domain)?;
        let marker = format!("xamppify:{domain}");
        content = content
            .lines()
            .filter(|line| !line.contains("xamppify:"))
            .collect::<Vec<_>>()
            .join("\n");
        content.push('\n');
        content.push_str(&format!("127.0.0.1 {domain} # {marker}\n"));
    } else {
        content = content
            .lines()
            .filter(|line| !line.contains("xamppify:"))
            .collect::<Vec<_>>()
            .join("\n");
    }
    std::fs::write(&path, content).map_err(|e| {
        format!(
            "Failed to update the hosts file: {e}. Try running the app as administrator."
        )
    })
}

pub fn remove_hosts_entry(_domain: &str) -> Result<(), String> {
    set_hosts_entry(None)
}
