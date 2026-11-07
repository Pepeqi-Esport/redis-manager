use std::time::Duration;

pub const APP_TITLE: &str = "Redis Manager";
pub const WINDOW_INNER_SIZE: [f32; 2] = [720.0, 820.0];
pub const WINDOW_MIN_INNER_SIZE: [f32; 2] = [640.0, 800.0];

pub const DEFAULT_REDIS_PORT: u16 = 6379;
pub const PING_TIMEOUT: Duration = Duration::from_millis(400);
pub const RUNTIME_DIR: &str = "runtime/redis";

pub const RUN_VALUE_NAME: &str = "RedisManager";
pub const AUTOSTART_ARG: &str = "--autostart";
pub const LEGACY_LAUNCHER_NAME: &str = "autostart.vbs";

/// StartupApproved: first byte 2 = enabled, 3 = disabled (Windows Settings > Startup apps).
#[cfg(windows)]
pub const STARTUP_APPROVED_ENABLED: [u8; 12] = [2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

pub const BUNDLE_MARKER_NAME: &str = ".bundle_id";
pub const AUTOSTART_LOG_NAME: &str = "autostart.log";

pub const STATUS_REFRESH_INTERVAL_MS: u64 = 800;
pub const UI_REPAINT_INTERVAL_MS: u64 = 250;
