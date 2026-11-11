use crate::services::redis_ctl_service::RedisStatus;
use crate::services::worker_service::UiAction;
use crate::utils::ThemePreference;
use crate::views::components::widgets::{accent_toggle, section_card, toggle_row};
use crate::views::pages::HomePage;
use eframe::egui::{self, Color32, RichText, Stroke, Vec2};

pub fn settings_panel(ui: &mut egui::Ui, app: &mut HomePage) {
    let p = app.palette();
    section_card(ui, p, "Settings", |ui| {
        toggle_row(ui, p, "Redis service", |ui| {
            let running = matches!(app.status, RedisStatus::Running { .. });
            let can_toggle = !app.busy
                && app.paths.as_ref().is_some_and(|paths| paths.exists())
                && !matches!(app.status, RedisStatus::MissingInstall);
            let mut on = running;
            ui.add_enabled_ui(can_toggle, |ui| {
                if accent_toggle(ui, p, &mut on) {
                    if on {
                        app.request(UiAction::Start);
                    } else {
                        app.request(UiAction::Stop);
                    }
                }
            });
        });

        ui.add_space(12.0);

        toggle_row(ui, p, "Launch on startup", |ui| {
            let mut enabled = app.autostart_enabled;
            ui.add_enabled_ui(!app.autostart_busy && app.paths.is_some(), |ui| {
                if accent_toggle(ui, p, &mut enabled) {
                    app.request(UiAction::SetAutostart(enabled));
                    app.autostart_enabled = !enabled;
                }
            });
        });

        ui.add_space(12.0);

        ui.horizontal(|ui| {
            ui.label(RichText::new("Theme").size(14.0).color(p.text));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let mut selected = app.theme_preference;
                egui::ComboBox::from_id_salt("theme_preference")
                    .selected_text(selected.label())
                    .width(110.0)
                    .show_ui(ui, |ui| {
                        for pref in ThemePreference::ALL {
                            ui.selectable_value(&mut selected, pref, pref.label());
                        }
                    });
                if selected != app.theme_preference {
                    app.set_theme_preference(ui.ctx(), selected);
                }
            });
        });

        ui.add_space(12.0);
        ui.label(
            RichText::new("Start Redis automatically after Windows startup.")
                .size(12.0)
                .color(p.text_dim),
        );

        ui.add_space(14.0);
        if ui
            .add_enabled(
                !app.busy,
                egui::Button::new(
                    RichText::new("Refresh status")
                        .size(13.0)
                        .color(p.accent),
                )
                .fill(Color32::TRANSPARENT)
                .stroke(Stroke::new(1.0_f32, p.accent_border))
                .min_size(Vec2::new(ui.available_width(), 34.0)),
            )
            .clicked()
        {
            app.request(UiAction::Refresh);
        }
    });
}
