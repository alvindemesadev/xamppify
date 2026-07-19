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
