use crate::redis_ctl::RedisPaths;
use std::path::{Path, PathBuf};

#[cfg(windows)]
use winreg::enums::{HKEY_CURRENT_USER, KEY_ALL_ACCESS, KEY_READ, KEY_SET_VALUE, REG_BINARY};
#[cfg(windows)]
use winreg::{RegKey, RegValue};

const RUN_VALUE_NAME: &str = "RedisManager";
const AUTOSTART_ARG: &str = "--autostart";
const LEGACY_LAUNCHER_NAME: &str = "autostart.vbs";

/// StartupApproved: first byte 2 = enabled, 3 = disabled (Windows Settings > Startup apps).
#[cfg(windows)]
const STARTUP_APPROVED_ENABLED: [u8; 12] = [2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnableOutcome {
    Created,
    Recreated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisableOutcome {
    Removed,
    AlreadyAbsent,
}

/// Whether Redis is registered to start at Windows logon.
pub fn is_enabled() -> bool {
    entry_exists()
}

pub fn enable(paths: &RedisPaths) -> Result<EnableOutcome, String> {
    #[cfg(windows)]
    {
        if !paths.exists() {
            return Err("Redis runtime is incomplete.".into());
        }

        // OFF -> ON: if an entry already exists, remove it then recreate.
        let existed = entry_exists() || legacy_launcher_exists(paths);
        if existed {
            remove_entry(Some(paths))?;
        }

        create_entry()?;

        if !entry_exists() {
            return Err("Failed to register start at login (entry missing after create).".into());
        }

        Ok(if existed {
            EnableOutcome::Recreated
        } else {
            EnableOutcome::Created
        })
    }

    #[cfg(not(windows))]
    {
        let _ = paths;
        Err("Autostart is only supported on Windows.".into())
    }
}

pub fn disable(paths: Option<&RedisPaths>) -> Result<DisableOutcome, String> {
    #[cfg(windows)]
    {
        // ON -> OFF: if the entry still exists, remove it.
        let existed = entry_exists() || paths.map(legacy_launcher_exists).unwrap_or(false);
        if !existed {
            return Ok(DisableOutcome::AlreadyAbsent);
        }

        remove_entry(paths)?;

        if entry_exists() {
            return Err("Failed to remove start at login (entry still present).".into());
        }

        Ok(DisableOutcome::Removed)
    }

    #[cfg(not(windows))]
    {
        let _ = paths;
        Err("Autostart is only supported on Windows.".into())
    }
}

fn entry_exists() -> bool {
    #[cfg(windows)]
    {
        read_run_value().is_some()
    }
    #[cfg(not(windows))]
    {
        false
    }
}

fn legacy_launcher_exists(paths: &RedisPaths) -> bool {
    legacy_launcher_path(&paths.root).is_file()
}

#[cfg(windows)]
fn create_entry() -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| format!("Failed to resolve app path: {e}"))?;
    let exe = normalize_path(&exe);
    if !exe.is_file() {
        return Err(format!("App executable not found:\n{}", exe.display()));
    }

    // Run the same binary headlessly at logon. Avoids fragile VBS + Cygwin path issues.
    let command = format!("\"{}\" {AUTOSTART_ARG}", path_for_windows(&exe));

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (run_key, _) = hkcu
        .create_subkey_with_flags(
            r"Software\Microsoft\Windows\CurrentVersion\Run",
            KEY_ALL_ACCESS,
        )
        .map_err(|e| format!("Failed to open Run registry key: {e}"))?;

    run_key
        .set_value(RUN_VALUE_NAME, &command)
        .map_err(|e| format!("Failed to write startup entry: {e}"))?;

    // Keep the entry enabled in Windows Startup Apps settings.
    let (approved_key, _) = hkcu
        .create_subkey_with_flags(
            r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run",
            KEY_ALL_ACCESS,
        )
        .map_err(|e| format!("Failed to open StartupApproved key: {e}"))?;

    approved_key
        .set_raw_value(
            RUN_VALUE_NAME,
            &RegValue {
                vtype: REG_BINARY,
                bytes: STARTUP_APPROVED_ENABLED.to_vec(),
            },
        )
        .map_err(|e| format!("Failed to enable startup approval: {e}"))?;

    Ok(())
}

fn remove_entry(paths: Option<&RedisPaths>) -> Result<(), String> {
    #[cfg(windows)]
    {
        if entry_exists() {
            let hkcu = RegKey::predef(HKEY_CURRENT_USER);
            let run_key = hkcu
                .open_subkey_with_flags(
                    r"Software\Microsoft\Windows\CurrentVersion\Run",
                    KEY_SET_VALUE,
                )
                .map_err(|e| format!("Failed to open Run registry key: {e}"))?;

            run_key
                .delete_value(RUN_VALUE_NAME)
                .map_err(|e| format!("Failed to delete startup entry: {e}"))?;
        }

        // Best-effort: clear StartupApproved so Settings UI stays consistent.
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        if let Ok(approved_key) = hkcu.open_subkey_with_flags(
            r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run",
            KEY_SET_VALUE,
        ) {
            let _ = approved_key.delete_value(RUN_VALUE_NAME);
        }

        if let Some(paths) = paths {
            let launcher = legacy_launcher_path(&paths.root);
            if launcher.is_file() {
                std::fs::remove_file(&launcher)
                    .map_err(|e| format!("Failed to delete legacy autostart launcher: {e}"))?;
            }
        }

        Ok(())
    }

    #[cfg(not(windows))]
    {
        let _ = paths;
        Ok(())
    }
}

#[cfg(windows)]
fn read_run_value() -> Option<String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run_key = hkcu
        .open_subkey_with_flags(r"Software\Microsoft\Windows\CurrentVersion\Run", KEY_READ)
        .ok()?;
    run_key.get_value::<String, _>(RUN_VALUE_NAME).ok()
}

fn legacy_launcher_path(redis_root: &Path) -> PathBuf {
    redis_root.join(LEGACY_LAUNCHER_NAME)
}

fn normalize_path(path: &Path) -> PathBuf {
    path.canonicalize()
        .unwrap_or_else(|_| path.to_path_buf())
}

fn path_for_windows(path: &Path) -> String {
    let mut s = path.display().to_string().replace('/', "\\");
    // canonicalize() on Windows may prefix \\?\ which can break Run entries.
    if let Some(stripped) = s.strip_prefix(r"\\?\") {
        s = stripped.to_string();
    }
    s
}
