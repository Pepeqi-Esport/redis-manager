use crate::services::autostart_service;
use crate::services::worker_service::{UiAction, WorkerMsg};
use crate::views::pages::HomePage;
use std::time::Instant;

impl HomePage {
    pub fn request(&mut self, action: UiAction) {
        match &action {
            UiAction::Refresh => {
                let _ = self.action_tx.send(action);
            }
            UiAction::Start | UiAction::Stop => {
                if self.busy {
                    return;
                }
                self.busy = true;
                self.last_message = match action {
                    UiAction::Start => "Starting Redis…".into(),
                    UiAction::Stop => "Stopping Redis…".into(),
                    _ => self.last_message.clone(),
                };
                let _ = self.action_tx.send(action);
            }
            UiAction::SetAutostart(enabled) => {
                if self.autostart_busy {
                    return;
                }
                self.autostart_busy = true;
                self.last_message = if *enabled {
                    "Enabling start at login…".into()
                } else {
                    "Disabling start at login…".into()
                };
                let _ = self.action_tx.send(action);
            }
        }
    }

    pub(super) fn poll_worker(&mut self) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                WorkerMsg::Status(status) => {
                    self.status = status;
                    self.last_refresh = Instant::now();
                }
                WorkerMsg::Done {
                    ok,
                    message,
                    status,
                } => {
                    self.busy = false;
                    self.status = status;
                    self.last_message = if ok {
                        message
                    } else {
                        format!("Error: {message}")
                    };
                    self.last_refresh = Instant::now();
                }
                WorkerMsg::AutostartDone {
                    ok,
                    enabled,
                    message,
                } => {
                    self.autostart_busy = false;
                    if ok {
                        self.autostart_enabled = enabled;
                        self.last_message = message;
                    } else {
                        self.last_message = format!("Error: {message}");
                        self.autostart_enabled = autostart_service::is_enabled();
                    }
                }
            }
        }
    }
}
