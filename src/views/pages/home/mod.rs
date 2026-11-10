mod actions;
mod theme;

use crate::config::load_theme_preference;
use crate::helpers::{load_texture, window_icon};
use crate::helpers::update_helper::UpdatePrefs;
use crate::services::autostart_service;
use crate::services::redis_ctl_service::{self, RedisPaths, RedisStatus};
use crate::services::runtime_bundle_service;
use crate::services::worker_service::{self, UiAction, WorkerMsg};
use crate::utils::{
    apply_theme, configure_style, ThemePreference, STATUS_REFRESH_INTERVAL_MS, UI_REPAINT_INTERVAL_MS,
};
use crate::views::components::update::{self as update_ui, UpdateDialog, UpdateMsg};
use crate::views::layouts::main_layout;
use eframe::egui::{self, TextureHandle};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

pub struct HomePage {
    pub paths: Option<RedisPaths>,
    pub status: RedisStatus,
    pub busy: bool,
    pub autostart_enabled: bool,
    pub autostart_busy: bool,
    pub theme_preference: ThemePreference,
    pub(crate) resolved_dark: bool,
    pub(crate) last_message: String,
    pub(crate) last_refresh: Instant,
    pub(crate) rx: Receiver<WorkerMsg>,
    pub(crate) action_tx: Sender<UiAction>,
    pub(crate) logo: TextureHandle,
    pub(crate) update_prefs: UpdatePrefs,
    pub(crate) update_dialog: Option<UpdateDialog>,
    pub(crate) update_busy: bool,
    pub(crate) update_startup_started: bool,
    pub(crate) update_tx: Sender<UpdateMsg>,
    pub(crate) update_rx: Receiver<UpdateMsg>,
}

impl HomePage {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        configure_style(&cc.egui_ctx);

        let theme_preference = load_theme_preference();
        apply_theme(&cc.egui_ctx, theme_preference);
        let resolved_dark = theme_preference.resolves_dark();

        cc.egui_ctx
            .send_viewport_cmd(egui::ViewportCommand::Icon(Some(std::sync::Arc::new(
                window_icon(),
            ))));

        let logo = load_texture(&cc.egui_ctx);
        let (update_tx, update_rx) = update_ui::update_channel();

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
            theme_preference,
            resolved_dark,
            last_message: extract_error
                .map(|e| format!("Error: failed to extract Redis runtime: {e}"))
                .unwrap_or_default(),
            last_refresh: Instant::now(),
            rx,
            action_tx,
            logo,
            update_prefs: update_ui::load_prefs(),
            update_dialog: None,
            update_busy: false,
            update_startup_started: false,
            update_tx,
            update_rx,
        }
    }
}

impl eframe::App for HomePage {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_worker();
        self.poll_update_messages();
        self.sync_system_theme(ctx);

        if self.last_refresh.elapsed() >= Duration::from_millis(STATUS_REFRESH_INTERVAL_MS)
            && !self.busy
        {
            self.request(UiAction::Refresh);
            self.last_refresh = Instant::now();
        }

        let repaint_ms = if self.update_busy || self.update_dialog.is_some() {
            100
        } else {
            UI_REPAINT_INTERVAL_MS
        };
        ctx.request_repaint_after(Duration::from_millis(repaint_ms));

        main_layout(ctx, self);
        self.ensure_startup_update_check(ctx);
        self.show_update_dialog(ctx);
    }
}
