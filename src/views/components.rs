use crate::helpers::lerp_u8;
use crate::services::redis_ctl_service::{RedisPaths, RedisStatus};
use crate::services::worker_service::UiAction;
use crate::views::app::RedisManagerApp;
use eframe::egui::{self, Color32, RichText, Sense, Stroke, Vec2};

pub fn settings_card(ui: &mut egui::Ui, app: &mut RedisManagerApp) {
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

pub fn status_card(ui: &mut egui::Ui, status: &RedisStatus, paths: &RedisPaths) {
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
