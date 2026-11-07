use crate::services::redis_ctl_service::{RedisPaths, RedisStatus};
use crate::utils::DEFAULT_REDIS_PORT;
use crate::views::pages::HomePage;
use crate::views::components::widgets::{overview_row, section_card};
use eframe::egui;

pub fn overview_panel(ui: &mut egui::Ui, app: &HomePage, paths: Option<&RedisPaths>) {
    let p = app.palette();
    let (host, port, pid) = match &app.status {
        RedisStatus::Running { pid, port } => (
            "127.0.0.1".to_string(),
            port.to_string(),
            pid.map(|id| id.to_string())
                .unwrap_or_else(|| "n/a".into()),
        ),
        _ => ("127.0.0.1".into(), DEFAULT_REDIS_PORT.to_string(), "—".into()),
    };

    let runtime = paths
        .map(|paths| paths.root.display().to_string())
        .unwrap_or_else(|| "Not found".into());
    let runtime_short = if runtime.len() > 28 {
        format!("…{}", &runtime[runtime.len().saturating_sub(24)..])
    } else {
        runtime
    };
    let host_port = format!("{host}:{port}");

    section_card(ui, p, "Overview", |ui| {
        overview_row(ui, p, "Host", &host_port);
        ui.add_space(10.0);
        overview_row(ui, p, "PID", &pid);
        ui.add_space(10.0);
        overview_row(ui, p, "Runtime", &runtime_short);
    });
}
