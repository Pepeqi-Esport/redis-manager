use crate::views::components::{
    app_header, footer_status, live_status_panel, overview_panel, service_panel,
    service_panel_missing, settings_panel,
};
use crate::views::pages::HomePage;
use eframe::egui;

pub fn main_layout(ctx: &egui::Context, page: &mut HomePage) {
    let bg = page.palette().bg;
    egui::CentralPanel::default()
        .frame(
            egui::Frame::new()
                .fill(bg)
                .inner_margin(egui::Margin::symmetric(20, 18)),
        )
        .show(ctx, |ui| {
            let logo = page.logo.clone();
            app_header(ui, page, &logo);
            ui.add_space(18.0);

            let available = ui.available_width();
            let gap = 14.0;
            let col_w = ((available - gap) * 0.5).max(260.0);
            let card_gap = 28.0;

            ui.horizontal_top(|ui| {
                ui.allocate_ui_with_layout(
                    egui::vec2(col_w, ui.available_height()),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        if page.paths.is_some() {
                            service_panel(ui, page);
                        } else {
                            service_panel_missing(ui, page.palette());
                        }
                        ui.add_space(card_gap);
                        settings_panel(ui, page);
                    },
                );

                ui.add_space(gap);

                ui.allocate_ui_with_layout(
                    egui::vec2(col_w, ui.available_height()),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        live_status_panel(ui, page);
                        ui.add_space(card_gap);
                        overview_panel(ui, page, page.paths.as_ref());
                    },
                );
            });

            ui.add_space(14.0);
            footer_status(ui, page, &page.last_message.clone());
        });
}
