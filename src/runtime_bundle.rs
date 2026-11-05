use rust_embed::Embed;
use std::fs;
use std::path::{Path, PathBuf};

/// Redis runtime files compiled into the binary.
/// Extracted on first launch so only the .exe needs to be distributed.
#[derive(Embed)]
#[folder = "runtime/redis/"]
#[exclude = "*.pid"]
#[exclude = "dump.rdb"]
#[exclude = "appendonly.aof"]
#[exclude = "autostart.vbs"]
#[exclude = "temp-*.rdb"]
struct RedisRuntime;

const MARKER_NAME: &str = ".bundle_id";

/// Directory where embedded Redis files are extracted.
pub fn install_dir() -> Result<PathBuf, String> {
    let base = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("TEMP").map(PathBuf::from))
        .ok_or_else(|| "LOCALAPPDATA/TEMP not set".to_string())?;

    Ok(base.join("RedisManager").join("runtime").join("redis"))
}

/// Ensure embedded runtime exists on disk. Re-extracts when the app version changes.
pub fn ensure_extracted() -> Result<PathBuf, String> {
    let dir = install_dir()?;
    let marker = dir.join(MARKER_NAME);
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

        fs::write(&dest, file.data.as_ref())
            .map_err(|e| format!("Failed to extract {}: {e}", dest.display()))?;

        // Clear read-only bit if present so future updates can overwrite.
        let _ = clear_readonly(&dest);
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
        fs::set_permissions(path, perms)?;
    }
    Ok(())
}
