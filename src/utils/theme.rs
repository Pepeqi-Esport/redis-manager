use eframe::egui::{self, Color32, CornerRadius, FontFamily, FontId, Stroke, TextStyle, Vec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemePreference {
    System,
    Light,
    Dark,
}

impl ThemePreference {
    pub const ALL: [Self; 3] = [Self::System, Self::Light, Self::Dark];

    pub fn label(self) -> &'static str {
        match self {
            Self::System => "System",
            Self::Light => "Light",
            Self::Dark => "Dark",
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "system" => Some(Self::System),
            "light" => Some(Self::Light),
            "dark" => Some(Self::Dark),
            _ => None,
        }
    }

    pub fn resolves_dark(self) -> bool {
        match self {
            Self::Light => false,
            Self::Dark => true,
            Self::System => system_is_dark(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub bg: Color32,
    pub card: Color32,
    pub card_hover: Color32,
    pub border: Color32,
    pub border_soft: Color32,
    pub text: Color32,
    pub text_muted: Color32,
    pub text_dim: Color32,
    pub accent: Color32,
    pub accent_soft: Color32,
    pub accent_border: Color32,
    pub success: Color32,
    pub danger: Color32,
    pub danger_soft: Color32,
    pub warning: Color32,
    pub warning_soft: Color32,
    pub toggle_off: Color32,
    pub toggle_on: Color32,
    pub pill_bg: Color32,
    pub row: Color32,
}

impl Palette {
    pub const DARK: Self = Self {
        bg: Color32::from_rgb(18, 18, 18),
        card: Color32::from_rgb(28, 28, 28),
        card_hover: Color32::from_rgb(34, 34, 34),
        border: Color32::from_rgb(48, 48, 48),
        border_soft: Color32::from_rgb(40, 40, 40),
        text: Color32::from_rgb(236, 236, 236),
        text_muted: Color32::from_rgb(150, 150, 150),
        text_dim: Color32::from_rgb(110, 110, 110),
        accent: Color32::from_rgb(232, 90, 90),
        accent_soft: Color32::from_rgb(56, 28, 28),
        accent_border: Color32::from_rgb(120, 48, 48),
        success: Color32::from_rgb(56, 200, 110),
        danger: Color32::from_rgb(232, 90, 90),
        danger_soft: Color32::from_rgb(48, 24, 24),
        warning: Color32::from_rgb(220, 170, 70),
        warning_soft: Color32::from_rgb(48, 40, 20),
        toggle_off: Color32::from_rgb(70, 70, 70),
        toggle_on: Color32::from_rgb(232, 90, 90),
        pill_bg: Color32::from_rgb(32, 40, 34),
        row: Color32::from_rgb(32, 32, 32),
    };

    pub const LIGHT: Self = Self {
        bg: Color32::from_rgb(244, 245, 247),
        card: Color32::from_rgb(255, 255, 255),
        card_hover: Color32::from_rgb(248, 249, 251),
        border: Color32::from_rgb(226, 230, 236),
        border_soft: Color32::from_rgb(236, 239, 243),
        text: Color32::from_rgb(24, 28, 36),
        text_muted: Color32::from_rgb(110, 118, 132),
        text_dim: Color32::from_rgb(150, 156, 168),
        accent: Color32::from_rgb(220, 56, 44),
        accent_soft: Color32::from_rgb(255, 236, 234),
        accent_border: Color32::from_rgb(235, 180, 175),
        success: Color32::from_rgb(22, 163, 114),
        danger: Color32::from_rgb(220, 70, 70),
        danger_soft: Color32::from_rgb(255, 235, 235),
        warning: Color32::from_rgb(200, 140, 40),
        warning_soft: Color32::from_rgb(255, 244, 220),
        toggle_off: Color32::from_rgb(200, 206, 216),
        toggle_on: Color32::from_rgb(220, 56, 44),
        pill_bg: Color32::from_rgb(220, 245, 234),
        row: Color32::from_rgb(248, 249, 251),
    };

    pub fn for_dark(dark: bool) -> Self {
        if dark {
            Self::DARK
        } else {
            Self::LIGHT
        }
    }
}

pub fn system_is_dark() -> bool {
    #[cfg(windows)]
    {
        use winreg::enums::HKEY_CURRENT_USER;
        use winreg::RegKey;
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        if let Ok(key) =
            hkcu.open_subkey(r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize")
        {
            // 0 = dark apps, 1 = light apps
            if let Ok(value) = key.get_value::<u32, _>("AppsUseLightTheme") {
                return value == 0;
            }
        }
        true
    }
    #[cfg(not(windows))]
    {
        true
    }
}

pub fn configure_style(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = Vec2::new(10.0, 10.0);
    style.spacing.button_padding = Vec2::new(14.0, 8.0);
    style.spacing.window_margin = egui::Margin::same(0);
    style.interaction.show_tooltips_only_when_still = true;

    style.text_styles.insert(
        TextStyle::Heading,
        FontId::new(24.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Body,
        FontId::new(14.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Button,
        FontId::new(13.5, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Small,
        FontId::new(12.0, FontFamily::Proportional),
    );
    ctx.set_style(style);
}

pub fn apply_theme(ctx: &egui::Context, preference: ThemePreference) {
    let dark = preference.resolves_dark();
    let p = Palette::for_dark(dark);

    let mut visuals = if dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };

    visuals.panel_fill = p.bg;
    visuals.window_fill = p.card;
    visuals.extreme_bg_color = p.bg;
    visuals.faint_bg_color = p.row;
    visuals.widgets.noninteractive.bg_fill = p.card;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, p.text);
    visuals.widgets.inactive.bg_fill = p.row;
    visuals.widgets.inactive.weak_bg_fill = p.row;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, p.text);
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, p.border);
    visuals.widgets.hovered.bg_fill = p.card_hover;
    visuals.widgets.hovered.weak_bg_fill = p.card_hover;
    visuals.widgets.hovered.bg_stroke = Stroke::new(
        1.0_f32,
        if dark {
            Color32::from_rgb(70, 70, 70)
        } else {
            Color32::from_rgb(210, 216, 226)
        },
    );
    visuals.widgets.active.bg_fill = if dark {
        Color32::from_rgb(40, 40, 40)
    } else {
        Color32::from_rgb(235, 238, 244)
    };
    visuals.widgets.active.bg_stroke = Stroke::new(
        1.0_f32,
        if dark {
            Color32::from_rgb(80, 80, 80)
        } else {
            Color32::from_rgb(200, 208, 220)
        },
    );
    visuals.selection.bg_fill = p.accent_soft;
    visuals.selection.stroke = Stroke::new(1.0_f32, p.accent);
    visuals.override_text_color = Some(p.text);
    visuals.window_corner_radius = CornerRadius::same(14);
    visuals.menu_corner_radius = CornerRadius::same(10);
    visuals.widgets.noninteractive.corner_radius = CornerRadius::same(10);
    visuals.widgets.inactive.corner_radius = CornerRadius::same(10);
    visuals.widgets.hovered.corner_radius = CornerRadius::same(10);
    visuals.widgets.active.corner_radius = CornerRadius::same(10);

    ctx.set_visuals(visuals);
}
