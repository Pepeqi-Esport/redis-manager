use eframe::egui;

pub fn load_app_icon() -> egui::IconData {
    // Prefer the square icon generated from logo.png. Windows window icons are
    // unreliable with large non-square PNGs (logo.png is 567x486).
    let bytes = include_bytes!("../../assets/images/logo/icon-256.png");
    let image = image::load_from_memory(bytes)
        .expect("icon-256.png must be valid")
        .into_rgba8();
    let (width, height) = image.dimensions();
    egui::IconData {
        rgba: image.into_raw(),
        width,
        height,
    }
}
