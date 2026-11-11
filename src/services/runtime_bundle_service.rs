use crate::helpers::path_beside_exe;
use crate::utils::{BUNDLE_MARKER_NAME, RUNTIME_DIR};
use rust_embed::Embed;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Redis runtime files compiled into the binary.
/// Extracted on first launch beside the .exe (`runtime/redis`).
#[derive(Embed)]
#[folder = "runtime/redis/"]
#[exclude = "*.pid"]
#[exclude = "dump.rdb"]
#[exclude = "appendonly.aof"]
#[exclude = "autostart.vbs"]
#[exclude = "temp-*.rdb"]
struct RedisRuntime;

/// Directory where embedded Redis files are extracted (next to the exe).
pub fn install_dir() -> Result<PathBuf, String> {
    path_beside_exe(RUNTIME_DIR)
}

/// Ensure embedded runtime exists on disk. Re-extracts when the app version changes.
/// Locked files (e.g. redis-server.exe while Redis is running) are skipped so sync
/// still succeeds; they will refresh on the next launch after Redis is stopped.
pub fn ensure_extracted() -> Result<PathBuf, String> {
    let dir = install_dir()?;
    let marker = dir.join(BUNDLE_MARKER_NAME);
    let bundle_id = bundle_id();

    let up_to_date = dir.join("redis-server.exe").is_file()
        && dir.join("redis.conf").is_file()
        && fs::read_to_string(&marker)
            .map(|s| s.trim() == bundle_id)
            .unwrap_or(false);

    if up_to_date {
        return Ok(dir);
    }

    fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create runtime dir {}: {e}", dir.display()))?;

    for path in RedisRuntime::iter() {
        let Some(file) = RedisRuntime::get(path.as_ref()) else {
            continue;
        };

        let dest = dir.join(path.as_ref());
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create {}: {e}", parent.display()))?;
        }

        let _ = clear_readonly(&dest);
        match fs::write(&dest, file.data.as_ref()) {
            Ok(()) => {}
            Err(e) if is_sharing_violation(&e) && dest.is_file() => {
                // Keep the running binary; update remaining files and marker.
                continue;
            }
            Err(e) => {
                return Err(format!("Failed to extract {}: {e}", dest.display()));
            }
        }
    }

    fs::write(&marker, &bundle_id)
        .map_err(|e| format!("Failed to write bundle marker: {e}"))?;

    Ok(dir)
}

fn bundle_id() -> String {
    // Bump when embedded runtime contents change meaningfully.
    format!("{}:{}", env!("CARGO_PKG_VERSION"), RedisRuntime::iter().count())
}

fn clear_readonly(path: &Path) -> std::io::Result<()> {
    let mut perms = fs::metadata(path)?.permissions();
    if perms.readonly() {
        perms.set_readonly(false);
    }
    fs::set_permissions(path, perms)?;
    Ok(())
}

fn is_sharing_violation(err: &io::Error) -> bool {
    matches!(
        err.raw_os_error(),
        Some(32) | Some(33) // ERROR_SHARING_VIOLATION / ERROR_LOCK_VIOLATION
    )
}
