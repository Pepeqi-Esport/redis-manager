use crate::services::redis_ctl_service::RedisStatus;
use crate::views::pages::HomePage;
use crate::views::components::widgets::section_card;
use eframe::egui::{self, RichText};

pub fn live_status_panel(ui: &mut egui::Ui, app: &HomePage) {
    let p = app.palette();
    let (title, big, foot, color) = match &app.status {
        RedisStatus::Running { port, .. } => (
            "Redis status",
            "RUNNING",
            format!("Redis · Port {port}"),
            p.success,
        ),
        RedisStatus::Stopped => (
            "Redis status",
            "STOPPED",
            "Redis · Idle".into(),
            p.danger,
        ),
        RedisStatus::MissingInstall => (
            "Redis status",
            "MISSING",
            "Runtime not found".into(),
            p.warning,
        ),
    };

    section_card(ui, p, "Live status", |ui| {
        ui.set_min_height(280.0);
        ui.label(RichText::new(title).size(13.0).color(p.text_muted));
        ui.add_space(10.0);
        ui.label(RichText::new(big).size(34.0).strong().color(color));
        ui.add_space(8.0);
        ui.label(RichText::new(foot).size(12.0).color(p.text_dim));

        if app.busy || app.autostart_busy {
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(
                    RichText::new("Working…")
                        .size(12.5)
                        .color(p.text_muted),
                );
            });
        }
    });
}
