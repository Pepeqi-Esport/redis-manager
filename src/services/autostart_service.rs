use crate::helpers::{normalize_path, path_for_windows};
use crate::services::redis_ctl_service::{self, RedisPaths};
use crate::services::runtime_bundle_service;
use crate::utils::{AUTOSTART_ARG, LEGACY_LAUNCHER_NAME, RUN_VALUE_NAME};
use std::path::{Path, PathBuf};

#[cfg(windows)]
use crate::utils::STARTUP_APPROVED_ENABLED;
#[cfg(windows)]
use winreg::enums::{HKEY_CURRENT_USER, KEY_ALL_ACCESS, KEY_READ, KEY_SET_VALUE, REG_BINARY};
#[cfg(windows)]
use winreg::{RegKey, RegValue};

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

/// Silent login start: extract runtime and start Redis without opening the UI.
pub fn run_autostart() -> Result<(), String> {
    runtime_bundle_service::ensure_extracted()?;
    let paths = RedisPaths::discover()
        .ok_or_else(|| "Redis runtime not found after extract.".to_string())?;
    redis_ctl_service::start(&paths)?;
    Ok(())
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

pub fn outcome_message(outcome: EnableOutcome) -> String {
    match outcome {
        EnableOutcome::Created => {
            "Start at login enabled. Redis will start after Windows login.".into()
        }
        EnableOutcome::Recreated => {
            "Existing entry removed and recreated. Start at login is on.".into()
        }
    }
}

pub fn disable_message(outcome: DisableOutcome) -> String {
    match outcome {
        DisableOutcome::Removed => {
            "Start at login disabled. Startup entry removed.".into()
        }
        DisableOutcome::AlreadyAbsent => {
            "Start at login already off (entry not found).".into()
        }
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
                bytes: STARTUP_APPROVED_ENABLED.to_vec().into(),
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
