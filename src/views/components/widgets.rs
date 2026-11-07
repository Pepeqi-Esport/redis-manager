use crate::helpers::lerp_u8;
use crate::utils::Palette;
use eframe::egui::{self, Color32, CornerRadius, RichText, Sense, Stroke, Vec2};

pub fn section_card(
    ui: &mut egui::Ui,
    p: Palette,
    title: &str,
    add_contents: impl FnOnce(&mut egui::Ui),
) {
    egui::Frame::new()
        .fill(p.card)
        .stroke(Stroke::new(1.0_f32, p.border))
        .corner_radius(CornerRadius::same(16))
        .inner_margin(egui::Margin::symmetric(16, 14))
        .show(ui, |ui| {
            ui.label(RichText::new(title).size(12.5).color(p.text_muted));
            ui.add_space(10.0);
            add_contents(ui);
        });
}

pub fn metric_row(ui: &mut egui::Ui, p: Palette, label: &str, value: &str, highlighted: bool) {
    let (fill, stroke) = if highlighted {
        (p.accent_soft, p.accent_border)
    } else {
        (p.row, p.border_soft)
    };

    egui::Frame::new()
        .fill(fill)
        .stroke(Stroke::new(1.0_f32, stroke))
        .corner_radius(CornerRadius::same(12))
        .inner_margin(egui::Margin::symmetric(12, 10))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(label).size(13.5).color(p.text_muted));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(value)
                            .size(15.0)
                            .strong()
                            .color(p.text),
                    );
                });
            });
        });
}

pub fn toggle_row(
    ui: &mut egui::Ui,
    p: Palette,
    label: &str,
    trailing: impl FnOnce(&mut egui::Ui),
) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).size(14.0).color(p.text));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), trailing);
    });
}

pub fn overview_row(ui: &mut egui::Ui, p: Palette, label: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(label)
                .size(13.5)
                .strong()
                .color(p.text),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(RichText::new(value).size(13.5).color(p.text_muted));
        });
    });
}

/// Returns `true` when the user toggled the switch.
pub fn accent_toggle(ui: &mut egui::Ui, p: Palette, on: &mut bool) -> bool {
    let desired = Vec2::new(46.0, 26.0);
    let (rect, mut response) = ui.allocate_exact_size(desired, Sense::click());

    let toggled = response.clicked();
    if toggled {
        *on = !*on;
        response.mark_changed();
    }

    let how_on = ui.ctx().animate_bool_responsive(response.id, *on);
    let enabled = ui.is_enabled();
    let bg = if enabled {
        Color32::from_rgb(
            lerp_u8(p.toggle_off.r(), p.toggle_on.r(), how_on),
            lerp_u8(p.toggle_off.g(), p.toggle_on.g(), how_on),
            lerp_u8(p.toggle_off.b(), p.toggle_on.b(), how_on),
        )
    } else {
        Color32::from_rgb(55, 55, 55)
    };

    ui.painter()
        .rect_filled(rect, CornerRadius::same(13), bg);

    let radius = 9.0;
    let x = egui::lerp((rect.left() + 13.0)..=(rect.right() - 13.0), how_on);
    ui.painter()
        .circle_filled(egui::pos2(x, rect.center().y), radius, Color32::WHITE);

    if response.hovered() && enabled {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    toggled
}
