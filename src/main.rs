// Hide the empty console window on Windows (GUI app).
#![cfg_attr(windows, windows_subsystem = "windows")]

mod config;
mod helpers;
mod services;
mod utils;
mod views;

use crate::helpers::{ensure_sidecar_config_files, window_icon, write_autostart_log};
use crate::services::autostart_service;
use crate::utils::{
    fonts, APP_TITLE, AUTOSTART_ARG, WINDOW_INNER_SIZE, WINDOW_MIN_INNER_SIZE,
};
use crate::views::HomePage;
use eframe::egui;

fn main() -> eframe::Result<()> {
    ensure_sidecar_config_files();

    // Silent login start: register this via HKCU Run as `redis-manager.exe --autostart`.
    if std::env::args().skip(1).any(|arg| arg == AUTOSTART_ARG) {
        match autostart_service::run_autostart() {
            Ok(()) => {
                let _ = write_autostart_log("autostart ok");
                std::process::exit(0);
            }
            Err(err) => {
                let _ = write_autostart_log(&format!("autostart failed: {err}"));
                std::process::exit(1);
            }
        }
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(WINDOW_INNER_SIZE)
            .with_min_inner_size(WINDOW_MIN_INNER_SIZE)
            .with_title(APP_TITLE)
            .with_icon(window_icon()),
        ..Default::default()
    };

    eframe::run_native(
        APP_TITLE,
        options,
        Box::new(|cc| {
            fonts::install(&cc.egui_ctx);
            Ok(Box::new(HomePage::new(cc)))
        }),
    )
}
