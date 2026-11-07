use crate::services::redis_ctl_service::RedisStatus;
use crate::utils::{Palette, DEFAULT_REDIS_PORT};
use crate::views::pages::HomePage;
use crate::views::components::widgets::{metric_row, section_card};
use eframe::egui;

pub fn service_panel(ui: &mut egui::Ui, app: &HomePage) {
    let p = app.palette();
    let (port, pid_text, running) = match &app.status {
        RedisStatus::Running { pid, port } => (
            *port,
            pid.map(|id| id.to_string())
                .unwrap_or_else(|| "n/a".into()),
            true,
        ),
        RedisStatus::Stopped => (DEFAULT_REDIS_PORT, "—".into(), false),
        RedisStatus::MissingInstall => (DEFAULT_REDIS_PORT, "—".into(), false),
    };

    section_card(ui, p, "Service", |ui| {
        ui.set_min_height(280.0);
        metric_row(ui, p, "Redis", if running { "On" } else { "Off" }, true);
        ui.add_space(8.0);
        metric_row(ui, p, "Port", &port.to_string(), false);
        ui.add_space(8.0);
        metric_row(ui, p, "PID", &pid_text, false);
    });
}

pub fn service_panel_missing(ui: &mut egui::Ui, p: Palette) {
    egui::Frame::new()
        .fill(p.card)
        .stroke(egui::Stroke::new(1.0_f32, p.border))
        .corner_radius(egui::CornerRadius::same(16))
        .inner_margin(egui::Margin::symmetric(16, 14))
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new("Service")
                    .size(12.5)
                    .color(p.text_muted),
            );
            ui.add_space(10.0);
            ui.label(
                egui::RichText::new("Redis runtime not found.\nEmbedded runtime failed to extract.")
                    .size(13.0)
                    .color(p.danger),
            );
        });
}
