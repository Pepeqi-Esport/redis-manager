use crate::services::redis_ctl_service::RedisStatus;
use crate::utils::Palette;
use crate::views::pages::HomePage;
use eframe::egui::{self, Color32, CornerRadius, Image, RichText, Sense, Stroke, TextureHandle, Vec2};

pub fn app_header(ui: &mut egui::Ui, app: &HomePage, logo: &TextureHandle) {
    let p = app.palette();
    ui.horizontal(|ui| {
        ui.add(Image::new(logo).fit_to_exact_size(Vec2::splat(42.0)));
        ui.add_space(10.0);
        ui.vertical(|ui| {
            ui.add_space(1.0);
            ui.label(
                RichText::new("Redis Manager")
                    .size(22.0)
                    .strong()
                    .color(p.text),
            );
            ui.label(
                RichText::new("Local Redis control")
                    .size(12.5)
                    .color(p.text_muted),
            );
        });

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            connection_pill(ui, &app.status, p);
        });
    });
}

fn connection_pill(ui: &mut egui::Ui, status: &RedisStatus, p: Palette) {
    let (label, fg, bg, border) = match status {
        RedisStatus::Running { .. } => ("Running", p.success, p.pill_bg, Color32::from_rgb(40, 70, 48)),
        RedisStatus::Stopped => ("Stopped", p.danger, p.danger_soft, p.accent_border),
        RedisStatus::MissingInstall => (
            "Incomplete",
            p.warning,
            p.warning_soft,
            Color32::from_rgb(90, 70, 30),
        ),
    };

    egui::Frame::new()
        .fill(bg)
        .stroke(Stroke::new(1.0_f32, border))
        .corner_radius(CornerRadius::same(20))
        .inner_margin(egui::Margin::symmetric(12, 6))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let (dot, _) = ui.allocate_exact_size(Vec2::splat(10.0), Sense::hover());
                ui.painter().circle_filled(dot.center(), 4.0, fg);
                ui.label(RichText::new(label).size(13.0).strong().color(fg));
            });
        });
}

pub fn footer_status(ui: &mut egui::Ui, app: &HomePage, message: &str) {
    let p = app.palette();
    let text = if message.is_empty() {
        "Ready".to_string()
    } else {
        message.to_string()
    };
    let color = if message.starts_with("Error") {
        p.danger
    } else {
        p.text_dim
    };
    ui.label(RichText::new(format!("Synced · {text}")).size(12.0).color(color));
}
