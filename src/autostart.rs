use crate::redis_ctl::RedisPaths;
use std::path::{Path, PathBuf};

#[cfg(windows)]
use winreg::enums::{HKEY_CURRENT_USER, KEY_ALL_ACCESS, KEY_READ, KEY_SET_VALUE};
#[cfg(windows)]
use winreg::RegKey;

const RUN_VALUE_NAME: &str = "RedisManager";
const LAUNCHER_NAME: &str = "autostart.vbs";

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

/// Whether Redis is registered to start at Windows logon (Herd-style).
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
        let existed = entry_exists() || launcher_exists(paths);
        if existed {
            remove_entry(Some(paths))?;
        }

        create_entry(paths)?;

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
        let existed = entry_exists() || paths.map(launcher_exists).unwrap_or(false);
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

fn launcher_exists(paths: &RedisPaths) -> bool {
    launcher_path(&paths.root).is_file()
}

#[cfg(windows)]
fn create_entry(paths: &RedisPaths) -> Result<(), String> {
    let launcher = write_launcher(paths)?;
    let command = format!("wscript.exe //nologo \"{}\"", path_for_windows(&launcher));

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

        if let Some(paths) = paths {
            let launcher = launcher_path(&paths.root);
            if launcher.is_file() {
                std::fs::remove_file(&launcher)
                    .map_err(|e| format!("Failed to delete autostart launcher: {e}"))?;
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

fn launcher_path(redis_root: &Path) -> PathBuf {
    redis_root.join(LAUNCHER_NAME)
}

fn write_launcher(paths: &RedisPaths) -> Result<PathBuf, String> {
    let root = normalize_path(&paths.root);
    let server = normalize_path(&paths.server);
    let conf = normalize_path(&paths.conf);

    let root_s = escape_vbs(&path_for_windows(&root));
    let server_s = escape_vbs(&path_for_windows(&server));
    let conf_s = escape_vbs(&path_for_windows(&conf));

    // WindowStyle 0 = hidden; do not wait for process exit.
    let script = format!(
        r#"Set sh = CreateObject("WScript.Shell")
sh.CurrentDirectory = "{root_s}"
sh.Run """{server_s}"" ""{conf_s}""", 0, False
"#
    );

    let path = launcher_path(&root);
    std::fs::write(&path, script)
        .map_err(|e| format!("Failed to write autostart launcher: {e}"))?;
    Ok(path)
}

fn normalize_path(path: &Path) -> PathBuf {
    path.canonicalize()
        .unwrap_or_else(|_| path.to_path_buf())
}

fn path_for_windows(path: &Path) -> String {
    let mut s = path.display().to_string().replace('/', "\\");
    // canonicalize() on Windows may prefix \\?\ which breaks wscript/VBS.
    if let Some(stripped) = s.strip_prefix(r"\\?\") {
        s = stripped.to_string();
    }
    s
}

fn escape_vbs(value: &str) -> String {
    value.replace('"', "\"\"")
}
