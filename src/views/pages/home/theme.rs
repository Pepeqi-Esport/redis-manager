use crate::config::save_theme_preference;
use crate::utils::{apply_theme, Palette, ThemePreference};
use crate::views::pages::HomePage;
use eframe::egui;

impl HomePage {
    pub fn palette(&self) -> Palette {
        Palette::for_dark(self.resolved_dark)
    }

    pub fn set_theme_preference(&mut self, ctx: &egui::Context, preference: ThemePreference) {
        self.theme_preference = preference;
        self.resolved_dark = preference.resolves_dark();
        apply_theme(ctx, preference);
        if let Err(err) = save_theme_preference(preference) {
            self.last_message = format!("Error: failed to save theme: {err}");
        }
    }

    pub(super) fn sync_system_theme(&mut self, ctx: &egui::Context) {
        if self.theme_preference != ThemePreference::System {
            return;
        }
        let dark = self.theme_preference.resolves_dark();
        if dark != self.resolved_dark {
            self.resolved_dark = dark;
            apply_theme(ctx, self.theme_preference);
        }
    }
}
