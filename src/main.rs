// Hide the empty console window on Windows (GUI app).
#![cfg_attr(windows, windows_subsystem = "windows")]

mod autostart;
mod redis_ctl;
mod runtime_bundle;

use eframe::egui::{self, Color32, RichText, Sense, Stroke, Vec2};
use redis_ctl::{RedisPaths, RedisStatus};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

fn main() -> eframe::Result<()> {
    // Silent login start: register this via HKCU Run as `redis-manager.exe --autostart`.
    if std::env::args().skip(1).any(|arg| arg == "--autostart") {
        match run_autostart() {
            Ok(()) => std::process::exit(0),
            Err(err) => {
                let _ = write_autostart_log(&format!("autostart failed: {err}"));
                std::process::exit(1);
            }
        }
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([440.0, 520.0])
            .with_min_inner_size([380.0, 500.0])
            .with_title("Redis Manager")
            .with_icon(load_app_icon()),
        ..Default::default()
    };

    eframe::run_native(
        "Redis Manager",
        options,
        Box::new(|cc| Ok(Box::new(RedisManagerApp::new(cc)))),
    )
}

fn run_autostart() -> Result<(), String> {
    runtime_bundle::ensure_extracted()?;
    let paths =
        RedisPaths::discover().ok_or_else(|| "Redis runtime not found after extract.".to_string())?;
    redis_ctl::start(&paths)?;
    let _ = write_autostart_log("autostart ok");
    Ok(())
}

fn write_autostart_log(message: &str) -> Result<(), String> {
    let dir = std::env::var_os("LOCALAPPDATA")
        .map(std::path::PathBuf::from)
        .map(|p| p.join("RedisManager"))
        .ok_or_else(|| "LOCALAPPDATA not set".to_string())?;

    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let line = format!("{} {}\n", chrono_like_timestamp(), message);
    std::fs::write(dir.join("autostart.log"), line).map_err(|e| e.to_string())?;
    Ok(())
}

fn chrono_like_timestamp() -> String {
    // Local wall-clock without extra deps (good enough for a tiny debug log).
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("unix:{secs}")
}

fn load_app_icon() -> egui::IconData {
    // Prefer the square icon generated from logo.png. Windows window icons are
    // unreliable with large non-square PNGs (logo.png is 567x486).
    let bytes = include_bytes!("../assets/images/logo/icon-256.png");
    let image = image::load_from_memory(bytes)
        .expect("icon-256.png must be valid")
        .into_rgba8();
    let (width, height) = image.dimensions();
    egui::IconData {
        rgba: image.into_raw(),
        width,
        height,
    }
}

enum UiAction {
    Start,
    Stop,
    Refresh,
    SetAutostart(bool),
}

enum WorkerMsg {
    Status(RedisStatus),
    Done {
        ok: bool,
        message: String,
        status: RedisStatus,
    },
    AutostartDone {
        ok: bool,
        enabled: bool,
        message: String,
    },
}

struct RedisManagerApp {
    paths: Option<RedisPaths>,
    status: RedisStatus,
    busy: bool,
    autostart_enabled: bool,
    autostart_busy: bool,
    last_message: String,
    last_refresh: Instant,
    rx: Receiver<WorkerMsg>,
    action_tx: Sender<UiAction>,
}

impl RedisManagerApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        configure_style(&cc.egui_ctx);
        // Re-apply icon after the window exists (more reliable on Windows).
        cc.egui_ctx
            .send_viewport_cmd(egui::ViewportCommand::Icon(Some(std::sync::Arc::new(
                load_app_icon(),
            ))));

        // Extract embedded Redis runtime (single-exe distribution).
        let extract_error = runtime_bundle::ensure_extracted().err();
        let paths = RedisPaths::discover();
        let initial_status = paths
            .as_ref()
            .map(redis_ctl::status)
            .unwrap_or(RedisStatus::MissingInstall);

        let (tx, rx) = mpsc::channel();
        let (action_tx, action_rx) = mpsc::channel::<UiAction>();

        let worker_paths = paths.clone();
        thread::spawn(move || worker_loop(worker_paths, action_rx, tx));

        Self {
            paths,
            status: initial_status,
            busy: false,
            autostart_enabled: autostart::is_enabled(),
            autostart_busy: false,
            last_message: extract_error
                .map(|e| format!("Error: failed to extract Redis runtime: {e}"))
                .unwrap_or_default(),
            last_refresh: Instant::now(),
            rx,
            action_tx,
        }
    }

    fn request(&mut self, action: UiAction) {
        match &action {
            UiAction::Refresh => {
                let _ = self.action_tx.send(action);
            }
            UiAction::Start | UiAction::Stop => {
                if self.busy {
                    return;
                }
                self.busy = true;
                self.last_message = match action {
                    UiAction::Start => "Starting Redis...".into(),
                    UiAction::Stop => "Stopping Redis...".into(),
                    _ => self.last_message.clone(),
                };
                let _ = self.action_tx.send(action);
            }
            UiAction::SetAutostart(enabled) => {
                if self.autostart_busy {
                    return;
                }
                self.autostart_busy = true;
                self.last_message = if *enabled {
                    "Enabling start at login...".into()
                } else {
                    "Disabling start at login...".into()
                };
                let _ = self.action_tx.send(action);
            }
        }
    }

    fn poll_worker(&mut self) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                WorkerMsg::Status(status) => {
                    self.status = status;
                    self.last_refresh = Instant::now();
                }
                WorkerMsg::Done {
                    ok,
                    message,
                    status,
                } => {
                    self.busy = false;
                    self.status = status;
                    self.last_message = if ok {
                        message
                    } else {
                        format!("Error: {message}")
                    };
                    self.last_refresh = Instant::now();
                }
                WorkerMsg::AutostartDone {
                    ok,
                    enabled,
                    message,
                } => {
                    self.autostart_busy = false;
                    if ok {
                        self.autostart_enabled = enabled;
                        self.last_message = message;
                    } else {
                        self.last_message = format!("Error: {message}");
                        self.autostart_enabled = autostart::is_enabled();
                    }
                }
            }
        }
    }
}

impl eframe::App for RedisManagerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_worker();

        if self.last_refresh.elapsed() >= Duration::from_millis(800) && !self.busy {
            self.request(UiAction::Refresh);
            self.last_refresh = Instant::now();
        }

        ctx.request_repaint_after(Duration::from_millis(250));

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(8.0);
            ui.vertical_centered(|ui| {
                ui.heading(RichText::new("Redis Manager").size(26.0).strong());
                ui.label(
                    RichText::new("Interactive local Redis control")
                        .color(Color32::from_rgb(150, 160, 175))
                        .size(13.0),
                );
            });

            ui.add_space(16.0);

            let paths = self.paths.clone();
            match paths.as_ref() {
                None => {
                    ui.colored_label(
                        Color32::from_rgb(220, 90, 90),
                        "Redis runtime not found.\nEmbedded runtime failed to extract.",
                    );
                }
                Some(paths) => {
                    status_card(ui, &self.status, paths);

                    ui.add_space(14.0);

                    let can_start = !self.busy
                        && matches!(self.status, RedisStatus::Stopped)
                        && paths.exists();
                    let can_stop =
                        !self.busy && matches!(self.status, RedisStatus::Running { .. });

                    ui.horizontal(|ui| {
                        ui.add_space((ui.available_width() - 260.0).max(0.0) * 0.5);

                        let start = egui::Button::new(
                            RichText::new("  Start  ")
                                .size(16.0)
                                .color(Color32::WHITE),
                        )
                        .fill(if can_start {
                            Color32::from_rgb(46, 160, 90)
                        } else {
                            Color32::from_rgb(70, 90, 75)
                        })
                        .min_size(Vec2::new(120.0, 40.0));

                        let stop = egui::Button::new(
                            RichText::new("  Stop  ")
                                .size(16.0)
                                .color(Color32::WHITE),
                        )
                        .fill(if can_stop {
                            Color32::from_rgb(190, 70, 70)
                        } else {
                            Color32::from_rgb(95, 70, 70)
                        })
                        .min_size(Vec2::new(130.0, 40.0));

                        if ui.add_enabled(can_start, start).clicked() {
                            self.request(UiAction::Start);
                        }
                        if ui.add_enabled(can_stop, stop).clicked() {
                            self.request(UiAction::Stop);
                        }
                    });

                    ui.add_space(12.0);

                    ui.horizontal(|ui| {
                        let refresh_width = if self.busy || self.autostart_busy {
                            190.0
                        } else {
                            130.0
                        };
                        ui.add_space((ui.available_width() - refresh_width).max(0.0) * 0.5);

                        if ui
                            .add_enabled(!self.busy, egui::Button::new("Refresh status"))
                            .clicked()
                        {
                            self.request(UiAction::Refresh);
                        }
                        if self.busy || self.autostart_busy {
                            ui.spinner();
                            ui.label("Working...");
                        }
                    });

                    ui.add_space(14.0);
                    settings_card(ui, self);

                    if !self.last_message.is_empty() {
                        ui.add_space(10.0);
                        let color = if self.last_message.starts_with("Error") {
                            Color32::from_rgb(230, 120, 100)
                        } else {
                            Color32::from_rgb(140, 190, 150)
                        };
                        ui.label(RichText::new(&self.last_message).color(color));
                    }
                }
            }
        });
    }
}

fn settings_card(ui: &mut egui::Ui, app: &mut RedisManagerApp) {
    egui::Frame::new()
        .fill(Color32::from_rgb(28, 32, 40))
        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(50, 58, 70)))
        .corner_radius(10.0)
        .inner_margin(egui::Margin::symmetric(16, 14))
        .show(ui, |ui| {
            ui.label(RichText::new("Settings").size(15.0).strong());
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("Start at login").size(14.0));
                    ui.label(
                        RichText::new("Run Redis automatically after Windows login")
                            .size(12.0)
                            .color(Color32::from_rgb(140, 150, 165)),
                    );
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let mut enabled = app.autostart_enabled;
                    ui.add_enabled_ui(!app.autostart_busy && app.paths.is_some(), |ui| {
                        if herd_toggle(ui, &mut enabled) {
                            app.request(UiAction::SetAutostart(enabled));
                            // Keep previous state until worker confirms.
                            app.autostart_enabled = !enabled;
                        }
                    });
                });
            });
        });
}

/// Returns `true` when the user toggled the switch.
fn herd_toggle(ui: &mut egui::Ui, on: &mut bool) -> bool {
    let desired = Vec2::new(46.0, 26.0);
    let (rect, mut response) = ui.allocate_exact_size(desired, Sense::click());

    let toggled = response.clicked();
    if toggled {
        *on = !*on;
        response.mark_changed();
    }

    let visuals = ui.style().interact(&response);
    let how_on = ui.ctx().animate_bool_responsive(response.id, *on);
    let bg = Color32::from_rgb(
        lerp_u8(70, 46, how_on),
        lerp_u8(78, 160, how_on),
        lerp_u8(92, 90, how_on),
    );

    ui.painter()
        .rect_filled(rect, 13.0, if ui.is_enabled() { bg } else { visuals.bg_fill });

    let radius = 9.0;
    let x = egui::lerp((rect.left() + 13.0)..=(rect.right() - 13.0), how_on);
    ui.painter().circle_filled(
        egui::pos2(x, rect.center().y),
        radius,
        Color32::WHITE,
    );

    toggled
}

fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t).round() as u8
}

fn outcome_message(outcome: autostart::EnableOutcome) -> String {
    match outcome {
        autostart::EnableOutcome::Created => {
            "Start at login enabled. Redis will start after Windows login.".into()
        }
        autostart::EnableOutcome::Recreated => {
            "Existing entry removed and recreated. Start at login is on.".into()
        }
    }
}

fn disable_message(outcome: autostart::DisableOutcome) -> String {
    match outcome {
        autostart::DisableOutcome::Removed => {
            "Start at login disabled. Startup entry removed.".into()
        }
        autostart::DisableOutcome::AlreadyAbsent => {
            "Start at login already off (entry not found).".into()
        }
    }
}

fn status_card(ui: &mut egui::Ui, status: &RedisStatus, paths: &RedisPaths) {
    let (label, detail, color) = match status {
        RedisStatus::Running { pid, port } => {
            let pid_text = pid
                .map(|p| format!("PID {p}"))
                .unwrap_or_else(|| "PID n/a".into());
            (
                "RUNNING",
                format!("Listening on 127.0.0.1:{port}  ·  {pid_text}"),
                Color32::from_rgb(56, 180, 110),
            )
        }
        RedisStatus::Stopped => (
            "NOT RUNNING",
            "Redis is not running".into(),
            Color32::from_rgb(200, 90, 90),
        ),
        RedisStatus::MissingInstall => (
            "INCOMPLETE",
            format!("Check install at {}", paths.root.display()),
            Color32::from_rgb(210, 150, 60),
        ),
    };

    let card_width = 360.0_f32.min(ui.available_width());
    ui.horizontal(|ui| {
        ui.add_space((ui.available_width() - card_width).max(0.0) * 0.5);
        ui.allocate_ui_with_layout(
            Vec2::new(card_width, ui.available_height()),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                egui::Frame::new()
                    .fill(Color32::from_rgb(28, 32, 40))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(50, 58, 70)))
                    .corner_radius(10.0)
                    .inner_margin(egui::Margin::symmetric(16, 14))
                    .show(ui, |ui| {
                        ui.set_min_width(card_width - 4.0);
                        ui.horizontal(|ui| {
                            let (rect, _resp) =
                                ui.allocate_exact_size(Vec2::splat(14.0), Sense::hover());
                            ui.painter().circle_filled(rect.center(), 6.0, color);

                            ui.add_space(8.0);
                            ui.vertical(|ui| {
                                ui.label(RichText::new(label).size(18.0).strong().color(color));
                                ui.label(
                                    RichText::new(detail)
                                        .size(13.0)
                                        .color(Color32::from_rgb(170, 178, 190)),
                                );
                            });
                        });
                    });
            },
        );
    });
}

fn configure_style(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = Vec2::new(10.0, 8.0);
    style.spacing.button_padding = Vec2::new(12.0, 8.0);
    ctx.set_style(style);

    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = Color32::from_rgb(18, 20, 26);
    visuals.window_fill = Color32::from_rgb(18, 20, 26);
    visuals.override_text_color = Some(Color32::from_rgb(220, 225, 232));
    ctx.set_visuals(visuals);
}

fn worker_loop(
    paths: Option<RedisPaths>,
    actions: Receiver<UiAction>,
    out: Sender<WorkerMsg>,
) {
    while let Ok(action) = actions.recv() {
        match action {
            UiAction::Refresh => {
                let status = paths
                    .as_ref()
                    .map(redis_ctl::status)
                    .unwrap_or(RedisStatus::MissingInstall);
                let _ = out.send(WorkerMsg::Status(status));
            }
            UiAction::Start => {
                let Some(paths) = paths.as_ref() else {
                    let _ = out.send(WorkerMsg::Done {
                        ok: false,
                        message: "Redis runtime not found (runtime/redis).".into(),
                        status: RedisStatus::MissingInstall,
                    });
                    continue;
                };
                let result = redis_ctl::start(paths);
                let status = redis_ctl::status(paths);
                let _ = out.send(match result {
                    Ok(()) => WorkerMsg::Done {
                        ok: true,
                        message: "Redis started successfully.".into(),
                        status,
                    },
                    Err(message) => WorkerMsg::Done {
                        ok: false,
                        message,
                        status,
                    },
                });
            }
            UiAction::Stop => {
                let Some(paths) = paths.as_ref() else {
                    let _ = out.send(WorkerMsg::Done {
                        ok: false,
                        message: "Redis runtime not found (runtime/redis).".into(),
                        status: RedisStatus::MissingInstall,
                    });
                    continue;
                };
                let result = redis_ctl::stop(paths);
                let status = redis_ctl::status(paths);
                let _ = out.send(match result {
                    Ok(()) => WorkerMsg::Done {
                        ok: true,
                        message: "Redis stopped successfully.".into(),
                        status,
                    },
                    Err(message) => WorkerMsg::Done {
                        ok: false,
                        message,
                        status,
                    },
                });
            }
            UiAction::SetAutostart(enabled) => {
                let result = if enabled {
                    match paths.as_ref() {
                        Some(paths) => autostart::enable(paths).map(|outcome| (true, outcome_message(outcome))),
                        None => Err("Redis runtime not found.".into()),
                    }
                } else {
                    autostart::disable(paths.as_ref()).map(|outcome| (false, disable_message(outcome)))
                };

                let _ = out.send(match result {
                    Ok((enabled, message)) => WorkerMsg::AutostartDone {
                        ok: true,
                        enabled,
                        message,
                    },
                    Err(message) => WorkerMsg::AutostartDone {
                        ok: false,
                        enabled: autostart::is_enabled(),
                        message,
                    },
                });
            }
        }
    }
}
