use crate::app::AppServices;
use crate::error::{AppError, PublicError};
use crate::models::{AppActionEvent, CaptureJobId, CaptureSource, FrontendAppAction};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

pub const APP_ACTION_AVAILABLE_EVENT: &str = "screenshot-ocr://app-action-available";
const RESERVATION_TIMEOUT: Duration = Duration::from_secs(10);
const QUIT_CLEANUP_TIMEOUT: Duration = Duration::from_secs(5);
const QUIT_POLL_INTERVAL: Duration = Duration::from_millis(50);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AppAction {
    ToggleCapture,
    StartCapture,
    CancelCapture,
    ShowMainWindow,
    Quit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActivationSource {
    Startup,
    SecondInstance,
    Tray,
    GlobalShortcut,
}

impl AppAction {
    pub fn from_argument(argument: Option<&str>) -> Option<Self> {
        match argument {
            Some("toggle") => Some(Self::ToggleCapture),
            Some("capture") => Some(Self::StartCapture),
            Some("cancel") => Some(Self::CancelCapture),
            Some("show") => Some(Self::ShowMainWindow),
            Some("quit") => Some(Self::Quit),
            _ => None,
        }
    }

    pub fn from_secondary_args(arguments: &[String]) -> Self {
        Self::from_argument(arguments.get(1).map(String::as_str)).unwrap_or(Self::ShowMainWindow)
    }
}

pub fn spawn_dispatch(app: AppHandle, action: AppAction, source: ActivationSource) {
    tauri::async_runtime::spawn(async move {
        if let Err(error) = dispatch(&app, action, source).await {
            let services = app.state::<AppServices>();
            let public = PublicError::from(error);
            services.runtime_diagnostics.record(public.code);
        }
    });
}

pub async fn dispatch(
    app: &AppHandle,
    action: AppAction,
    source: ActivationSource,
) -> Result<(), AppError> {
    let services = app.state::<AppServices>().inner().clone();
    match action {
        AppAction::ToggleCapture => {
            let state = services.state.lock().await;
            if state.active_job_id().is_some() {
                state.cancel_active();
                Ok(())
            } else {
                drop(state);
                request_frontend_capture(app, &services, capture_source(source)).await
            }
        }
        AppAction::StartCapture => {
            request_frontend_capture(app, &services, capture_source(source)).await
        }
        AppAction::CancelCapture => {
            services.state.lock().await.cancel_active();
            Ok(())
        }
        AppAction::ShowMainWindow => show_main_window(app),
        AppAction::Quit => {
            request_quit(app, services).await;
            Ok(())
        }
    }
}

async fn request_frontend_capture(
    app: &AppHandle,
    services: &AppServices,
    source: CaptureSource,
) -> Result<(), AppError> {
    let job_id = CaptureJobId::new();
    services.state.lock().await.reserve(job_id)?;
    let event = AppActionEvent {
        action: FrontendAppAction::StartCapture,
        job_id,
        source,
    };
    if let Err(error) = services.queue_app_action(event) {
        services.state.lock().await.expire_reservation(job_id);
        return Err(error);
    }
    if app.emit(APP_ACTION_AVAILABLE_EVENT, ()).is_err() {
        services.clear_app_action(job_id);
        services.state.lock().await.expire_reservation(job_id);
        return Err(AppError::Internal);
    }

    let services = services.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(RESERVATION_TIMEOUT).await;
        if services.state.lock().await.expire_reservation(job_id) {
            services.clear_app_action(job_id);
        }
    });
    Ok(())
}

async fn request_quit(app: &AppHandle, services: AppServices) {
    let wait_for_cleanup = services.state.lock().await.cancel_active();
    if !wait_for_cleanup {
        app.exit(0);
        return;
    }

    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        // Cancellation has already been requested; give the active job a bounded
        // grace period to unwind, then exit regardless. Waiting forever would let a
        // wedged capture make Quit do nothing at all. Helper processes are spawned
        // with `kill_on_drop`, so exiting still tears them down.
        let deadline = tokio::time::Instant::now() + QUIT_CLEANUP_TIMEOUT;
        while services.state.lock().await.active_job_id().is_some() {
            if tokio::time::Instant::now() >= deadline {
                break;
            }
            tokio::time::sleep(QUIT_POLL_INTERVAL).await;
        }
        app.exit(0);
    });
}

fn show_main_window(app: &AppHandle) -> Result<(), AppError> {
    let window = app.get_webview_window("main").ok_or(AppError::Internal)?;
    window.unminimize().map_err(|_| AppError::Internal)?;
    window.show().map_err(|_| AppError::Internal)?;
    window.set_focus().map_err(|_| AppError::Internal)
}

fn capture_source(source: ActivationSource) -> CaptureSource {
    match source {
        ActivationSource::Tray => CaptureSource::Tray,
        ActivationSource::GlobalShortcut => CaptureSource::Shortcut,
        ActivationSource::Startup | ActivationSource::SecondInstance => CaptureSource::CommandLine,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_only_supported_cli_actions() {
        assert_eq!(
            AppAction::from_argument(Some("toggle")),
            Some(AppAction::ToggleCapture)
        );
        assert_eq!(
            AppAction::from_argument(Some("capture")),
            Some(AppAction::StartCapture)
        );
        assert_eq!(
            AppAction::from_argument(Some("cancel")),
            Some(AppAction::CancelCapture)
        );
        assert_eq!(
            AppAction::from_argument(Some("show")),
            Some(AppAction::ShowMainWindow)
        );
        assert_eq!(
            AppAction::from_argument(Some("quit")),
            Some(AppAction::Quit)
        );
        assert_eq!(AppAction::from_argument(Some("--capture")), None);
        assert_eq!(AppAction::from_argument(None), None);
    }

    #[test]
    fn unknown_secondary_invocation_only_shows_the_window() {
        assert_eq!(
            AppAction::from_secondary_args(&["screenshot-ocr".into(), "unknown".into()]),
            AppAction::ShowMainWindow
        );
    }

    #[test]
    fn activation_sources_map_to_safe_capture_sources() {
        assert_eq!(capture_source(ActivationSource::Tray), CaptureSource::Tray);
        assert_eq!(
            capture_source(ActivationSource::GlobalShortcut),
            CaptureSource::Shortcut
        );
        assert_eq!(
            capture_source(ActivationSource::SecondInstance),
            CaptureSource::CommandLine
        );
    }
}
