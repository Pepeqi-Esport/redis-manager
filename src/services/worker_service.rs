use crate::services::autostart_service;
use crate::services::redis_ctl_service::{self, RedisPaths, RedisStatus};
use std::sync::mpsc::{Receiver, Sender};

pub enum UiAction {
    Start,
    Stop,
    Refresh,
    SetAutostart(bool),
}

pub enum WorkerMsg {
    Status(RedisStatus),
    Done {
        ok: bool,
        message: String,
        status: RedisStatus,
    },
    AutostartDone {
        ok: bool,
        enabled: bool,
        message: String,
    },
}

pub fn worker_loop(
    paths: Option<RedisPaths>,
    actions: Receiver<UiAction>,
    out: Sender<WorkerMsg>,
) {
    while let Ok(action) = actions.recv() {
        match action {
            UiAction::Refresh => {
                let status = paths
                    .as_ref()
                    .map(redis_ctl_service::status)
                    .unwrap_or(RedisStatus::MissingInstall);
                let _ = out.send(WorkerMsg::Status(status));
            }
            UiAction::Start => {
                let Some(paths) = paths.as_ref() else {
                    let _ = out.send(WorkerMsg::Done {
                        ok: false,
                        message: "Redis runtime not found (runtime/redis).".into(),
                        status: RedisStatus::MissingInstall,
                    });
                    continue;
                };
                let result = redis_ctl_service::start(paths);
                let status = redis_ctl_service::status(paths);
                let _ = out.send(match result {
                    Ok(()) => WorkerMsg::Done {
                        ok: true,
                        message: "Redis started successfully.".into(),
                        status,
                    },
                    Err(message) => WorkerMsg::Done {
                        ok: false,
                        message,
                        status,
                    },
                });
            }
            UiAction::Stop => {
                let Some(paths) = paths.as_ref() else {
                    let _ = out.send(WorkerMsg::Done {
                        ok: false,
                        message: "Redis runtime not found (runtime/redis).".into(),
                        status: RedisStatus::MissingInstall,
                    });
                    continue;
                };
                let result = redis_ctl_service::stop(paths);
                let status = redis_ctl_service::status(paths);
                let _ = out.send(match result {
                    Ok(()) => WorkerMsg::Done {
                        ok: true,
                        message: "Redis stopped successfully.".into(),
                        status,
                    },
                    Err(message) => WorkerMsg::Done {
                        ok: false,
                        message,
                        status,
                    },
                });
            }
            UiAction::SetAutostart(enabled) => {
                let result = if enabled {
                    match paths.as_ref() {
                        Some(paths) => autostart_service::enable(paths)
                            .map(|outcome| (true, autostart_service::outcome_message(outcome))),
                        None => Err("Redis runtime not found.".into()),
                    }
                } else {
                    autostart_service::disable(paths.as_ref())
                        .map(|outcome| (false, autostart_service::disable_message(outcome)))
                };

                let _ = out.send(match result {
                    Ok((enabled, message)) => WorkerMsg::AutostartDone {
                        ok: true,
                        enabled,
                        message,
                    },
                    Err(message) => WorkerMsg::AutostartDone {
                        ok: false,
                        enabled: autostart_service::is_enabled(),
                        message,
                    },
                });
            }
        }
    }
}
