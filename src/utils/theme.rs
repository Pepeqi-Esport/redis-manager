use eframe::egui::{self, Color32, Vec2};

pub fn configure_style(ctx: &egui::Context) {
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
