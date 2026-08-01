use crate::app::AppServices;
use crate::capture::EnvironmentProbe;
use crate::diagnostics::Diagnostics;
use crate::error::{AppError, PublicError};
use crate::models::{CaptureJobId, CaptureRequest, OcrResult};
use crate::ocr::TesseractEngine;
use crate::settings::AppSettings;
use tauri::{AppHandle, State, WebviewWindow};
use tauri_plugin_clipboard_manager::ClipboardExt;

#[tauri::command]
pub async fn start_capture(window: WebviewWindow, state: State<'_, AppServices>, request: CaptureRequest) -> Result<OcrResult, PublicError> {
    let _ = window.hide();
    let result = state.capture(request).await;
    let _ = window.show();
    let _ = window.set_focus();
    result.map_err(PublicError::from)
}

#[tauri::command]
pub async fn cancel_capture(state: State<'_, AppServices>, job_id: CaptureJobId) -> Result<(), PublicError> {
    state.state.lock().await.cancel(job_id).map_err(PublicError::from)
}

#[tauri::command]
pub fn copy_text(app: AppHandle, text: String) -> Result<(), PublicError> {
    if text.trim().is_empty() || text.len() > 1_000_000 {
        return Err(PublicError::from(AppError::OcrEmptyResult));
    }
    app.clipboard().write_text(text).map_err(|_| PublicError::from(AppError::ClipboardWriteFailed))
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppServices>) -> Result<AppSettings, PublicError> {
    state.settings.load().or_else(|error| if matches!(error, AppError::SettingsInvalid) { Ok(AppSettings::default()) } else { Err(error) }).map_err(PublicError::from)
}

#[tauri::command]
pub fn update_settings(state: State<'_, AppServices>, settings: AppSettings) -> Result<AppSettings, PublicError> {
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
    let languages = TesseractEngine::from_environment(&environment).and_then(|engine| engine.probe_english()).unwrap_or_default();
    Ok(Diagnostics::from_environment(&environment, languages))
}
