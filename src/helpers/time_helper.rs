use crate::helpers::path_beside_exe;
use crate::utils::AUTOSTART_LOG_NAME;

pub fn chrono_like_timestamp() -> String {
    // Local wall-clock without extra deps (good enough for a tiny debug log).
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("unix:{secs}")
}

pub fn write_autostart_log(message: &str) -> Result<(), String> {
    let path = path_beside_exe(AUTOSTART_LOG_NAME)?;
    let line = format!("{} {}\n", chrono_like_timestamp(), message);
    std::fs::write(path, line).map_err(|e| e.to_string())
}
