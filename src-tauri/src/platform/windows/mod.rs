mod composite;
mod geometry;
mod monitors;
mod ocr_runtime;
mod pixels;
mod selector;

use crate::cancellation::CancellationToken;
use crate::capture::CaptureBackend;
use crate::diagnostics::PlatformDiagnostics;
use crate::error::AppError;
use crate::models::CaptureBackendPreference;
use crate::ocr::OcrRuntime;
use crate::platform::PlatformServices;
use async_trait::async_trait;
use std::path::Path;

#[derive(Debug, Default)]
pub struct WindowsPlatform;

/// Report only a bounded capability result, never monitor geometry, display
/// names, hardware identifiers, or raw Win32 error messages.
fn capture_capability_summary(monitors_available: bool) -> &'static str {
    if monitors_available {
        "Windows native capture available; physical desktop validation pending"
    } else {
        "Windows native capture unavailable; physical desktop validation pending"
    }
}

#[async_trait]
impl PlatformServices for WindowsPlatform {
    async fn prepare_capture(
        &self,
        _preference: CaptureBackendPreference,
        cancellation: &CancellationToken,
    ) -> Result<Box<dyn CaptureBackend>, AppError> {
        // Historical Linux-specific backend preferences remain parseable when a
        // settings profile is migrated to Windows. Windows never attempts to
        // honor them: its adapter owns one native capture mechanism, and the
        // Windows UI will expose only truthful choices.
        cancellation.check()?;
        Ok(Box::new(pixels::NativeCapture))
    }

    fn resolve_ocr_runtime(&self) -> Result<OcrRuntime, AppError> {
        ocr_runtime::resolve_packaged_runtime()
    }

    // Preserve the existing Windows app-config ACL inheritance policy; do not
    // apply Unix permission bits or claim a reviewed Windows ACL policy.
    fn protect_settings_directory(&self, _path: &Path) -> Result<(), AppError> {
        Ok(())
    }

    fn protect_settings_file(&self, _path: &Path) -> Result<(), AppError> {
        Ok(())
    }

    async fn diagnostics(&self) -> Result<PlatformDiagnostics, AppError> {
        Ok(PlatformDiagnostics {
            platform: "windows".into(),
            os_release: "Windows".into(),
            desktop_environment: "Windows desktop".into(),
            session_type: "desktop".into(),
            capture_summary: capture_capability_summary(monitors::enumerate().is_ok()).into(),
            ocr_runtime: ocr_runtime::packaged_runtime_status(),
        })
    }

    fn direct_shortcut(&self) -> Option<&'static str> {
        Some("Control+Shift+O")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn migrated_linux_preferences_use_the_single_windows_backend() {
        let platform = WindowsPlatform;
        for preference in [
            CaptureBackendPreference::Auto,
            CaptureBackendPreference::Gnome,
            CaptureBackendPreference::Portal,
        ] {
            let cancellation = CancellationToken::new();
            assert!(platform
                .prepare_capture(preference, &cancellation)
                .await
                .is_ok());
        }
    }

    #[tokio::test]
    async fn cancelled_windows_capture_request_stays_cancelled() {
        let platform = WindowsPlatform;
        let cancellation = CancellationToken::new();
        cancellation.cancel();
        assert!(matches!(
            platform
                .prepare_capture(CaptureBackendPreference::Auto, &cancellation)
                .await,
            Err(AppError::CaptureCancelled)
        ));
    }

    #[test]
    fn windows_native_errors_have_stable_public_codes() {
        use crate::error::{ErrorCode, PublicError};
        for (error, expected) in [
            (AppError::CaptureCancelled, ErrorCode::CaptureCancelled),
            (
                AppError::CaptureBackendUnavailable,
                ErrorCode::CaptureBackendUnavailable,
            ),
            (AppError::CaptureFailed, ErrorCode::CaptureFailed),
            (
                AppError::CaptureImageInvalid,
                ErrorCode::CaptureImageInvalid,
            ),
            (AppError::CaptureTooLarge, ErrorCode::CaptureTooLarge),
            (
                AppError::OcrEngineUnavailable,
                ErrorCode::OcrEngineUnavailable,
            ),
            (AppError::OcrLanguageMissing, ErrorCode::OcrLanguageMissing),
            (
                AppError::ClipboardWriteFailed,
                ErrorCode::ClipboardWriteFailed,
            ),
        ] {
            let public = PublicError::from(error);
            assert_eq!(public.code, expected);
            assert!(public.retryable);
        }
    }

    #[test]
    fn capture_capability_summary_is_bounded_and_content_free() {
        assert_eq!(
            capture_capability_summary(true),
            "Windows native capture available; physical desktop validation pending"
        );
        assert_eq!(
            capture_capability_summary(false),
            "Windows native capture unavailable; physical desktop validation pending"
        );
    }

    #[tokio::test]
    async fn windows_capability_status_does_not_claim_native_capture_ready() {
        let diagnostics = WindowsPlatform.diagnostics().await.unwrap();
        assert_eq!(diagnostics.platform, "windows");
        assert!(diagnostics.capture_summary.contains("validation pending"));
        assert!(
            diagnostics.capture_summary == capture_capability_summary(true)
                || diagnostics.capture_summary == capture_capability_summary(false)
        );
        assert_eq!(WindowsPlatform.direct_shortcut(), Some("Control+Shift+O"));
    }
}
