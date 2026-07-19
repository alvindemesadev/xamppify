use crate::config_editor::{ConfigFile, IniSection};

#[tauri::command]
pub fn get_known_configs() -> Vec<ConfigFile> {
    crate::config_editor::known_configs()
}

#[tauri::command]
pub fn parse_ini_sections(content: String) -> Vec<IniSection> {
    crate::config_editor::parse_ini_sections(&content)
}
