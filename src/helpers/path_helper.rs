//! Paths next to the running executable (portable install layout).

use crate::helpers::update_helper::UpdatePrefs;
use std::path::{Path, PathBuf};

/// Directory that contains the running executable.
pub fn exe_dir() -> Result<PathBuf, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    exe.parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| "failed to resolve executable directory".to_string())
}

fn exe_dir_or_cwd() -> PathBuf {
    exe_dir().unwrap_or_else(|_| {
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    })
}

/// Path next to the executable (`exe_dir/relative`).
pub fn path_beside_exe(relative: impl AsRef<Path>) -> Result<PathBuf, String> {
    Ok(exe_dir()?.join(relative))
}

pub fn theme_json_path() -> PathBuf {
    exe_dir_or_cwd().join("theme.json")
}

pub fn update_json_path() -> PathBuf {
    exe_dir_or_cwd().join("update.json")
}

pub fn env_file_path() -> PathBuf {
    exe_dir_or_cwd().join(".env")
}

const DEFAULT_ENV: &str = "\
# Redis Manager local settings (beside the exe, or project root when using cargo run)
# Set to true to enable GitHub Releases update checks.
ENABLE_UPDATE_CHECK=false
";

/// Create required sidecar files beside the exe if they are missing
/// (`theme.json`, `update.json`, `.env`). Safe to call on every launch.
pub fn ensure_sidecar_config_files() {
    let theme = theme_json_path();
    if !theme.is_file() {
        let _ = std::fs::write(&theme, "{\n  \"theme\": \"system\"\n}\n");
    }
    let _ = UpdatePrefs::load();
    let env_path = env_file_path();
    if !env_path.is_file() {
        let _ = std::fs::write(&env_path, DEFAULT_ENV);
    }
}

/// Load `.env` from the current working directory and/or beside the exe.
/// Existing process env vars are never overwritten; earlier files win.
pub fn load_dotenv() {
    // Project-root `.env` first (`cargo run`), then portable `.env` beside the exe.
    let _ = dotenvy::dotenv();
    let beside = env_file_path();
    if beside.is_file() {
        let _ = dotenvy::from_path(&beside);
    }
}

pub fn normalize_path(path: &Path) -> PathBuf {
    path.canonicalize()
        .unwrap_or_else(|_| path.to_path_buf())
}

pub fn path_for_windows(path: &Path) -> String {
    let mut s = path.display().to_string().replace('/', "\\");
    // canonicalize() on Windows may prefix \\?\ which can break Run entries.
    if let Some(stripped) = s.strip_prefix(r"\\?\") {
        s = stripped.to_string();
    }
    s
}
