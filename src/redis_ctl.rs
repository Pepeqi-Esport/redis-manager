use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

const DEFAULT_PORT: u16 = 6379;
const PING_TIMEOUT: Duration = Duration::from_millis(400);
const RUNTIME_DIR: &str = "runtime/redis";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RedisStatus {
    Running { pid: Option<u32>, port: u16 },
    Stopped,
    MissingInstall,
}

#[derive(Debug, Clone)]
pub struct RedisPaths {
    pub root: PathBuf,
    pub server: PathBuf,
    pub cli: PathBuf,
    pub conf: PathBuf,
    pub pidfile: PathBuf,
}

impl RedisPaths {
    pub fn discover() -> Option<Self> {
        candidate_roots()
            .into_iter()
            .find(|root| root.join("redis-server.exe").is_file() && root.join("redis.conf").is_file())
            .map(Self::from_root)
    }

    fn from_root(root: PathBuf) -> Self {
        Self {
            server: root.join("redis-server.exe"),
            cli: root.join("redis-cli.exe"),
            conf: root.join("redis.conf"),
            pidfile: root.join("redis_6379.pid"),
            root,
        }
    }

    pub fn exists(&self) -> bool {
        self.server.is_file() && self.conf.is_file()
    }
}

fn candidate_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();

    // Extracted embedded runtime (single-exe distribution).
    if let Ok(dir) = crate::runtime_bundle::install_dir() {
        roots.push(dir);
    }

    // Optional override / portable layout beside the exe.
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            roots.push(exe_dir.join(RUNTIME_DIR));
            roots.push(exe_dir.to_path_buf());
            roots.push(exe_dir.join("../../").join(RUNTIME_DIR));
        }
    }

    // Source tree while developing with `cargo run`
    if let Ok(manifest) = std::env::var("CARGO_MANIFEST_DIR") {
        roots.push(PathBuf::from(manifest).join(RUNTIME_DIR));
    }

    if let Ok(cwd) = std::env::current_dir() {
        roots.push(cwd.join(RUNTIME_DIR));
    }

    let mut unique = Vec::new();
    for path in roots {
        let resolved = path.canonicalize().unwrap_or(path);
        if !unique.iter().any(|existing| existing == &resolved) {
            unique.push(resolved);
        }
    }
    unique
}

pub fn status(paths: &RedisPaths) -> RedisStatus {
    if !paths.exists() {
        return RedisStatus::MissingInstall;
    }

    if !is_port_open(DEFAULT_PORT) {
        return RedisStatus::Stopped;
    }

    RedisStatus::Running {
        pid: read_pid(&paths.pidfile),
        port: DEFAULT_PORT,
    }
}

pub fn start(paths: &RedisPaths) -> Result<(), String> {
    if !paths.exists() {
        return Err(format!(
            "redis-server not found at:\n{}",
            paths.root.display()
        ));
    }

    match status(paths) {
        RedisStatus::Running { .. } => return Ok(()),
        RedisStatus::MissingInstall => {
            return Err("Redis runtime installation is incomplete.".into());
        }
        RedisStatus::Stopped => {}
    }

    let mut cmd = Command::new(&paths.server);
    cmd.arg(&paths.conf)
        .current_dir(&paths.root)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);

    cmd.spawn()
        .map_err(|e| format!("Failed to start redis-server: {e}"))?;

    wait_until(
        |s| matches!(s, RedisStatus::Running { .. }),
        paths,
        Duration::from_secs(5),
    )
}

pub fn stop(paths: &RedisPaths) -> Result<(), String> {
    if !paths.exists() {
        return Err(format!(
            "redis-server not found at:\n{}",
            paths.root.display()
        ));
    }

    match status(paths) {
        RedisStatus::Stopped => return Ok(()),
        RedisStatus::MissingInstall => {
            return Err("Redis runtime installation is incomplete.".into());
        }
        RedisStatus::Running { pid, .. } => {
            if paths.cli.is_file() {
                let mut cmd = Command::new(&paths.cli);
                cmd.args([
                    "-h",
                    "127.0.0.1",
                    "-p",
                    &DEFAULT_PORT.to_string(),
                    "SHUTDOWN",
                    "NOSAVE",
                ])
                .current_dir(&paths.root)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null());

                #[cfg(windows)]
                cmd.creation_flags(CREATE_NO_WINDOW);

                let _ = cmd.status();
            }

            if wait_until(
                |s| matches!(s, RedisStatus::Stopped),
                paths,
                Duration::from_secs(3),
            )
            .is_ok()
            {
                return Ok(());
            }

            if let Some(pid) = pid.or_else(|| read_pid(&paths.pidfile)) {
                force_kill(pid)?;
                wait_until(
                    |s| matches!(s, RedisStatus::Stopped),
                    paths,
                    Duration::from_secs(2),
                )?;
                return Ok(());
            }

            Err("Redis is still running and could not be stopped.".into())
        }
    }
}

fn wait_until(
    predicate: impl Fn(RedisStatus) -> bool,
    paths: &RedisPaths,
    timeout: Duration,
) -> Result<(), String> {
    let started = Instant::now();
    loop {
        let current = status(paths);
        if predicate(current) {
            return Ok(());
        }
        if started.elapsed() >= timeout {
            return Err("Timed out waiting for Redis status change.".into());
        }
        std::thread::sleep(Duration::from_millis(120));
    }
}

fn is_port_open(port: u16) -> bool {
    let addr = format!("127.0.0.1:{port}");
    TcpStream::connect_timeout(&addr.parse().expect("valid socket addr"), PING_TIMEOUT).is_ok()
}

fn read_pid(pidfile: &Path) -> Option<u32> {
    let raw = std::fs::read_to_string(pidfile).ok()?;
    raw.trim().parse().ok()
}

fn force_kill(pid: u32) -> Result<(), String> {
    #[cfg(windows)]
    {
        let mut cmd = Command::new("taskkill");
        cmd.args(["/PID", &pid.to_string(), "/F", "/T"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(CREATE_NO_WINDOW);

        let status = cmd
            .status()
            .map_err(|e| format!("Failed to stop Redis process (PID {pid}): {e}"))?;

        if status.success() {
            Ok(())
        } else {
            Err(format!(
                "taskkill failed to stop Redis (PID {pid}, code {:?})",
                status.code()
            ))
        }
    }

    #[cfg(not(windows))]
    {
        let _ = pid;
        Err("Force kill is only supported on Windows.".into())
    }
}
