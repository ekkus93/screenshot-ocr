use crate::app::AppServices;
use crate::capture::{EnvironmentProbe, PortalScreenshotBackend};
use crate::diagnostics::Diagnostics;
use crate::error::{AppError, PublicError};
use crate::models::{AppActionEvent, CaptureJobId, CaptureRequest, CopyPolicy, OcrResult, OcrWarning};
use crate::ocr::TesseractEngine;
use crate::settings::{AppSettings, SettingsLoadResult};
use tauri::{AppHandle, Manager, State, WebviewWindow};
use tauri_plugin_clipboard_manager::ClipboardExt;

#[tauri::command]
pub async fn start_capture(
    window: WebviewWindow,
    state: State<'_, AppServices>,
    request: CaptureRequest,
) -> Result<OcrResult, PublicError> {
    let job_id = request.job_id;
    let copy_policy = request.copy_policy;
    window
        .hide()
        .map_err(|_| record_error(&state, AppError::Internal))?;
    let capture_result = state.capture(request).await;
    let result = match capture_result {
        Ok(mut result) if copy_policy == CopyPolicy::Immediate => {
            let copy_result = {
                let mut machine = state.state.lock().await;
                let copy_result = machine
                    .ensure_not_cancelled(job_id)
                    .and_then(|()| write_clipboard(window.app_handle(), &result.text));
                machine.finish(job_id);
                copy_result
            };
            match copy_result {
                Ok(()) => {
                    result.copied = true;
                    Ok(result)
                }
                Err(AppError::ClipboardWriteFailed) => {
                    let public = record_error(&state, AppError::ClipboardWriteFailed);
                    result.copied = false;
                    result.warnings.push(clipboard_failure_warning(&public));
                    Ok(result)
                }
                Err(error) => Err(record_error(&state, error)),
            }
        }
        Ok(result) => {
            state.state.lock().await.finish(job_id);
            Ok(result)
        }
        Err(error) => {
            state.state.lock().await.finish(job_id);
            Err(record_error(&state, error))
        }
    };
    let restore_result = restore_window(&window).map_err(|error| record_error(&state, error));
    match (result, restore_result) {
        (_, Err(error)) => Err(error),
        (result, Ok(())) => result,
    }
}

#[tauri::command]
pub async fn cancel_capture(
    state: State<'_, AppServices>,
    job_id: CaptureJobId,
) -> Result<(), PublicError> {
    state
        .state
        .lock()
        .await
        .cancel(job_id)
        .map_err(|error| record_error(&state, error))
}

#[tauri::command]
pub fn take_pending_app_action(state: State<'_, AppServices>) -> Option<AppActionEvent> {
    state.take_app_action()
}

#[tauri::command]
pub fn copy_text(
    app: AppHandle,
    state: State<'_, AppServices>,
    text: String,
) -> Result<(), PublicError> {
    write_clipboard(&app, &text).map_err(|error| record_error(&state, error))
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppServices>) -> Result<SettingsLoadResult, PublicError> {
    let result = state
        .settings
        .load_for_frontend()
        .map_err(|error| record_error(&state, error))?;
    if result.warning.is_some() {
        let public = PublicError::from(AppError::SettingsInvalid);
        state.runtime_diagnostics.record(public.code);
    }
    Ok(result)
}

#[tauri::command]
pub fn update_settings(
    state: State<'_, AppServices>,
    settings: AppSettings,
) -> Result<AppSettings, PublicError> {
    state
        .settings
        .save(&settings)
        .map_err(|error| record_error(&state, error))?;
    Ok(settings)
}

#[tauri::command]
pub fn reset_settings(state: State<'_, AppServices>) -> Result<AppSettings, PublicError> {
    state
        .settings
        .reset()
        .map_err(|error| record_error(&state, error))
}

#[tauri::command]
pub async fn get_diagnostics(state: State<'_, AppServices>) -> Result<Diagnostics, PublicError> {
    let environment = EnvironmentProbe::probe().map_err(|error| record_error(&state, error))?;
    let cancellation = crate::cancellation::CancellationToken::new();
    let languages = match TesseractEngine::from_environment(&environment) {
        Ok(engine) => engine.probe_english(&cancellation).await.unwrap_or_default(),
        Err(_) => Vec::new(),
    };
    let portal_summary = PortalScreenshotBackend::safe_capability_summary().await;
    Ok(Diagnostics::from_environment(
        &environment,
        languages,
        portal_summary,
        state.runtime_diagnostics.snapshot(),
    ))
}

fn record_error(state: &AppServices, error: AppError) -> PublicError {
    let public = PublicError::from(error);
    state.runtime_diagnostics.record(public.code);
    public
}

fn restore_window(window: &WebviewWindow) -> Result<(), AppError> {
    window.show().map_err(|_| AppError::Internal)?;
    window.set_focus().map_err(|_| AppError::Internal)
}

fn write_clipboard(app: &AppHandle, text: &str) -> Result<(), AppError> {
    validate_clipboard_text(text)?;
    app.clipboard()
        .write_text(text)
        .map_err(|_| AppError::ClipboardWriteFailed)
}

fn validate_clipboard_text(text: &str) -> Result<(), AppError> {
    if text.trim().is_empty() {
        return Err(AppError::OcrEmptyResult);
    }
    if text.len() > 1_000_000 {
        return Err(AppError::ClipboardWriteFailed);
    }
    Ok(())
}

fn clipboard_failure_warning(public: &PublicError) -> OcrWarning {
    OcrWarning {
        code: "clipboard_write_failed".into(),
        message: format!("{} {}", public.message, public.guidance),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn clipboard_validation_rejects_empty_and_oversized_text() {
        assert!(matches!(
            validate_clipboard_text(" \n"),
            Err(AppError::OcrEmptyResult)
        ));
        let oversized = "x".repeat(1_000_001);
        assert!(matches!(
            validate_clipboard_text(&oversized),
            Err(AppError::ClipboardWriteFailed)
        ));
    }

    #[test]
    fn clipboard_validation_preserves_code_whitespace() {
        assert!(validate_clipboard_text("  cargo test\n").is_ok());
    }

    #[test]
    fn clipboard_failure_warning_is_safe_and_recoverable() {
        let public = PublicError::from(AppError::ClipboardWriteFailed);
        let warning = clipboard_failure_warning(&public);
        assert_eq!(warning.code, "clipboard_write_failed");
        assert!(warning.message.contains("could not be copied"));
        assert!(!warning.message.contains("SYNTHETIC_SECRET_9f33"));
        assert!(!warning.message.contains("/tmp/"));
    }

    #[test]
    fn public_error_recording_updates_safe_runtime_state() {
        let directory = tempdir().expect("tempdir");
        let services = AppServices::new(directory.path().to_path_buf());
        let public = record_error(&services, AppError::TemporaryCleanupFailed);
        assert_eq!(public.code, crate::error::ErrorCode::TemporaryCleanupFailed);
        let snapshot = services.runtime_diagnostics.snapshot();
        assert_eq!(snapshot.cleanup_failure_count, 1);
        assert_eq!(
            snapshot.last_error_code,
            Some("temporary_cleanup_failed".into())
        );
    }
}
