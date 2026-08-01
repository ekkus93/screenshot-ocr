use crate::app::AppServices;
use crate::capture::EnvironmentProbe;
use crate::diagnostics::Diagnostics;
use crate::error::{AppError, PublicError};
use crate::models::{CaptureJobId, CaptureRequest, CopyPolicy, OcrResult};
use crate::ocr::TesseractEngine;
use crate::settings::AppSettings;
use tauri::{AppHandle, Manager, State, WebviewWindow};
use tauri_plugin_clipboard_manager::ClipboardExt;

#[tauri::command]
pub async fn start_capture(
    window: WebviewWindow,
    state: State<'_, AppServices>,
    request: CaptureRequest,
) -> Result<OcrResult, PublicError> {
    let copy_policy = request.copy_policy;
    window
        .hide()
        .map_err(|_| PublicError::from(AppError::Internal))?;
    let capture_result = state.capture(request).await;
    restore_window(&window).map_err(PublicError::from)?;
    let mut result = capture_result.map_err(PublicError::from)?;
    if copy_policy == CopyPolicy::Immediate {
        write_clipboard(window.app_handle(), &result.text).map_err(PublicError::from)?;
        result.copied = true;
    }
    Ok(result)
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
        .map_err(PublicError::from)
}

#[tauri::command]
pub fn take_startup_capture(state: State<'_, AppServices>) -> bool {
    state.take_startup_capture()
}

#[tauri::command]
pub fn copy_text(app: AppHandle, text: String) -> Result<(), PublicError> {
    write_clipboard(&app, &text).map_err(PublicError::from)
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppServices>) -> Result<AppSettings, PublicError> {
    state
        .settings
        .load()
        .or_else(|error| {
            if matches!(error, AppError::SettingsInvalid) {
                Ok(AppSettings::default())
            } else {
                Err(error)
            }
        })
        .map_err(PublicError::from)
}

#[tauri::command]
pub fn update_settings(
    state: State<'_, AppServices>,
    settings: AppSettings,
) -> Result<AppSettings, PublicError> {
    state.settings.save(&settings).map_err(PublicError::from)?;
    Ok(settings)
}

#[tauri::command]
pub fn reset_settings(state: State<'_, AppServices>) -> Result<AppSettings, PublicError> {
    state.settings.reset().map_err(PublicError::from)
}

#[tauri::command]
pub fn get_diagnostics() -> Result<Diagnostics, PublicError> {
    let environment = EnvironmentProbe::probe().map_err(PublicError::from)?;
    let languages = TesseractEngine::from_environment(&environment)
        .and_then(|engine| engine.probe_english())
        .unwrap_or_default();
    Ok(Diagnostics::from_environment(&environment, languages))
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
