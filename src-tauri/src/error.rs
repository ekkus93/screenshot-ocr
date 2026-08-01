use serde::Serialize;
use thiserror::Error;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    CaptureAlreadyActive,
    CaptureCancelled,
    CaptureBackendUnavailable,
    CapturePermissionDenied,
    CaptureProcessFailed,
    CaptureResultMissing,
    CaptureImageInvalid,
    CaptureTooLarge,
    TemporaryCleanupFailed,
    OcrEngineUnavailable,
    OcrLanguageMissing,
    OcrTimedOut,
    OcrFailed,
    OcrEmptyResult,
    ClipboardUnavailable,
    ClipboardWriteFailed,
    SettingsInvalid,
    SettingsWriteFailed,
    UnsupportedEnvironment,
    InternalError,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicError {
    pub code: ErrorCode,
    pub message: String,
    pub guidance: String,
    pub retryable: bool,
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("capture is already active")]
    CaptureAlreadyActive,
    #[error("capture was cancelled")]
    CaptureCancelled,
    #[error("capture backend is unavailable")]
    CaptureBackendUnavailable,
    #[error("capture helper failed")]
    CaptureProcessFailed,
    #[error("capture output is missing")]
    CaptureResultMissing,
    #[error("captured image is invalid")]
    CaptureImageInvalid,
    #[error("captured image exceeds safe limits")]
    CaptureTooLarge,
    #[error("temporary cleanup failed")]
    TemporaryCleanupFailed,
    #[error("OCR engine is unavailable")]
    OcrEngineUnavailable,
    #[error("OCR language is missing")]
    OcrLanguageMissing,
    #[error("OCR operation timed out")]
    OcrTimedOut,
    #[error("OCR operation failed")]
    OcrFailed,
    #[error("OCR result is empty")]
    OcrEmptyResult,
    #[error("clipboard write failed")]
    ClipboardWriteFailed,
    #[error("settings are invalid")]
    SettingsInvalid,
    #[error("settings write failed")]
    SettingsWriteFailed,
    #[error("operating environment is unsupported")]
    UnsupportedEnvironment,
    #[error("internal operation failed")]
    Internal,
}

impl From<AppError> for PublicError {
    fn from(error: AppError) -> Self {
        let (code, message, guidance, retryable) = match error {
            AppError::CaptureAlreadyActive => (ErrorCode::CaptureAlreadyActive, "A capture is already active.", "Finish or cancel the active selector before trying again.", true),
            AppError::CaptureCancelled => (ErrorCode::CaptureCancelled, "Capture was cancelled.", "Start another capture when ready. The clipboard was unchanged.", true),
            AppError::CaptureBackendUnavailable => (ErrorCode::CaptureBackendUnavailable, "No safe region-capture backend is available.", "Install gnome-screenshot or use a supported GNOME session.", true),
            AppError::CaptureProcessFailed => (ErrorCode::CaptureProcessFailed, "The screen-selection helper failed.", "Try again and inspect safe diagnostics if it continues.", true),
            AppError::CaptureResultMissing => (ErrorCode::CaptureResultMissing, "The selector did not produce an image.", "Try selecting a region again.", true),
            AppError::CaptureImageInvalid => (ErrorCode::CaptureImageInvalid, "The selected image could not be decoded safely.", "Select a different region and try again.", true),
            AppError::CaptureTooLarge => (ErrorCode::CaptureTooLarge, "The selected region is too large.", "Select a smaller region.", true),
            AppError::TemporaryCleanupFailed => (ErrorCode::TemporaryCleanupFailed, "A temporary capture could not be removed.", "Quit the application and inspect its runtime directory before continuing.", false),
            AppError::OcrEngineUnavailable => (ErrorCode::OcrEngineUnavailable, "Tesseract is not available.", "Install tesseract-ocr and tesseract-ocr-eng.", true),
            AppError::OcrLanguageMissing => (ErrorCode::OcrLanguageMissing, "English OCR data is missing.", "Install the tesseract-ocr-eng package.", true),
            AppError::OcrTimedOut => (ErrorCode::OcrTimedOut, "OCR took too long and was stopped.", "Select a smaller region and try again. The clipboard was unchanged.", true),
            AppError::OcrFailed => (ErrorCode::OcrFailed, "Tesseract could not recognize the selected region.", "Try a clearer or smaller region. The clipboard was unchanged.", true),
            AppError::OcrEmptyResult => (ErrorCode::OcrEmptyResult, "No readable text was found.", "Select an area containing larger, higher-contrast text. The clipboard was unchanged.", true),
            AppError::ClipboardWriteFailed => (ErrorCode::ClipboardWriteFailed, "The recognized text could not be copied.", "Keep the preview open and retry the copy action.", true),
            AppError::SettingsInvalid => (ErrorCode::SettingsInvalid, "The settings are invalid.", "Review the selected values or reset settings.", true),
            AppError::SettingsWriteFailed => (ErrorCode::SettingsWriteFailed, "Settings could not be saved.", "Check configuration-directory permissions and try again.", true),
            AppError::UnsupportedEnvironment => (ErrorCode::UnsupportedEnvironment, "This desktop environment is not supported.", "Use Ubuntu GNOME on Wayland or X11.", false),
            AppError::Internal => (ErrorCode::InternalError, "An internal operation failed.", "Restart the application and inspect safe diagnostics.", true),
        };
        Self { code, message: message.into(), guidance: guidance.into(), retryable }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_errors_never_include_sensitive_source_text() {
        let public = PublicError::from(AppError::OcrFailed);
        let json = serde_json::to_string(&public).expect("serialize public error");
        assert!(!json.contains("SYNTHETIC_SECRET_9f33"));
        assert!(!json.contains("/tmp/"));
    }
}
