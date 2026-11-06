use crate::utils::{
    configure_style, STATUS_REFRESH_INTERVAL_MS, UI_REPAINT_INTERVAL_MS,
};
use crate::services::autostart_service;
use crate::services::redis_ctl_service::{self, RedisPaths, RedisStatus};
use crate::services::runtime_bundle_service;
use crate::services::worker_service::{self, UiAction, WorkerMsg};
use crate::views::components::{settings_card, status_card};
use crate::views::icon::load_app_icon;
use eframe::egui::{self, Color32, RichText, Vec2};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

pub struct RedisManagerApp {
    pub paths: Option<RedisPaths>,
    status: RedisStatus,
    busy: bool,
    pub autostart_enabled: bool,
    pub autostart_busy: bool,
    last_message: String,
    last_refresh: Instant,
    rx: Receiver<WorkerMsg>,
    action_tx: Sender<UiAction>,
}

impl RedisManagerApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        configure_style(&cc.egui_ctx);
        // Re-apply icon after the window exists (more reliable on Windows).
        cc.egui_ctx
            .send_viewport_cmd(egui::ViewportCommand::Icon(Some(std::sync::Arc::new(
                load_app_icon(),
            ))));

        // Extract embedded Redis runtime (single-exe distribution).
        let extract_error = runtime_bundle_service::ensure_extracted().err();
        let paths = RedisPaths::discover();
        let initial_status = paths
            .as_ref()
            .map(redis_ctl_service::status)
            .unwrap_or(RedisStatus::MissingInstall);

        let (tx, rx) = mpsc::channel();
        let (action_tx, action_rx) = mpsc::channel::<UiAction>();

        let worker_paths = paths.clone();
        thread::spawn(move || worker_service::worker_loop(worker_paths, action_rx, tx));

        Self {
            paths,
            status: initial_status,
            busy: false,
            autostart_enabled: autostart_service::is_enabled(),
            autostart_busy: false,
            last_message: extract_error
                .map(|e| format!("Error: failed to extract Redis runtime: {e}"))
                .unwrap_or_default(),
            last_refresh: Instant::now(),
            rx,
            action_tx,
        }
    }

    pub fn request(&mut self, action: UiAction) {
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
                        self.autostart_enabled = autostart_service::is_enabled();
                    }
                }
            }
        }
    }
}

impl eframe::App for RedisManagerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_worker();

        if self.last_refresh.elapsed() >= Duration::from_millis(STATUS_REFRESH_INTERVAL_MS)
            && !self.busy
        {
            self.request(UiAction::Refresh);
            self.last_refresh = Instant::now();
        }

        ctx.request_repaint_after(Duration::from_millis(UI_REPAINT_INTERVAL_MS));

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
