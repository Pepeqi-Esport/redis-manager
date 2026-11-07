use crate::utils::ThemePreference;
use std::path::PathBuf;

const SETTINGS_FILE: &str = "settings.ini";

fn settings_path() -> Option<PathBuf> {
    std::env::var_os("LOCALAPPDATA").map(|base| {
        PathBuf::from(base)
            .join("RedisManager")
            .join(SETTINGS_FILE)
    })
}

pub fn load_theme_preference() -> ThemePreference {
    let Some(path) = settings_path() else {
        return ThemePreference::System;
    };
    let Ok(contents) = std::fs::read_to_string(path) else {
        return ThemePreference::System;
    };

    for line in contents.lines() {
        let line = line.trim();
        if let Some(value) = line.strip_prefix("theme=") {
            if let Some(pref) = ThemePreference::parse(value) {
                return pref;
            }
        }
    }

    ThemePreference::System
}

pub fn save_theme_preference(preference: ThemePreference) -> Result<(), String> {
    let path = settings_path().ok_or_else(|| "LOCALAPPDATA not set".to_string())?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let mut theme_written = false;
    let mut lines: Vec<String> = if path.exists() {
        std::fs::read_to_string(&path)
            .unwrap_or_default()
            .lines()
            .map(|line| {
                if line.trim().starts_with("theme=") {
                    theme_written = true;
                    format!("theme={}", preference.as_str())
                } else {
                    line.to_string()
                }
            })
            .collect()
    } else {
        Vec::new()
    };

    if !theme_written {
        lines.push(format!("theme={}", preference.as_str()));
    }

    let body = {
        let mut s = lines.join("\n");
        if !s.ends_with('\n') {
            s.push('\n');
        }
        s
    };
    std::fs::write(path, body).map_err(|e| e.to_string())
}
