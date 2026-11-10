use crate::helpers::theme_json_path;
use crate::utils::ThemePreference;

pub fn load_theme_preference() -> ThemePreference {
    let path = theme_json_path();
    let Ok(contents) = std::fs::read_to_string(&path) else {
        let _ = save_theme_preference(ThemePreference::System);
        return ThemePreference::System;
    };
    parse_theme_json(&contents).unwrap_or(ThemePreference::System)
}

pub fn save_theme_preference(preference: ThemePreference) -> Result<(), String> {
    let path = theme_json_path();
    let body = format!("{{\n  \"theme\": \"{}\"\n}}\n", preference.as_str());
    std::fs::write(path, body).map_err(|e| e.to_string())
}

fn parse_theme_json(contents: &str) -> Option<ThemePreference> {
    // Expect: { "theme": "system" | "light" | "dark" }
    let compact: String = contents.chars().filter(|c| !c.is_whitespace()).collect();
    let key = "\"theme\":\"";
    let start = compact.find(key)? + key.len();
    let end = compact[start..].find('"')? + start;
    ThemePreference::parse(&compact[start..end])
}
