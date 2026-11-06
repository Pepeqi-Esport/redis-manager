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
    let dir = std::env::var_os("LOCALAPPDATA")
        .map(std::path::PathBuf::from)
        .map(|p| p.join("RedisManager"))
        .ok_or_else(|| "LOCALAPPDATA not set".to_string())?;

    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let line = format!("{} {}\n", chrono_like_timestamp(), message);
    std::fs::write(dir.join(AUTOSTART_LOG_NAME), line).map_err(|e| e.to_string())?;
    Ok(())
}
