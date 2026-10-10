mod geometry;
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

#[derive(Debug, Default)]
pub struct WindowsPlatform;

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

    async fn diagnostics(&self) -> Result<PlatformDiagnostics, AppError> {
        Ok(PlatformDiagnostics {
            platform: "windows".into(),
            os_release: "Windows".into(),
            desktop_environment: "Windows desktop".into(),
            session_type: "desktop".into(),
            capture_summary:
                "Windows native region selector installed; physical desktop validation pending"
                    .into(),
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

    #[tokio::test]
    async fn windows_capability_status_does_not_claim_native_capture_ready() {
        let diagnostics = WindowsPlatform.diagnostics().await.unwrap();
        assert_eq!(diagnostics.platform, "windows");
        assert!(diagnostics.capture_summary.contains("validation pending"));
        assert_eq!(WindowsPlatform.direct_shortcut(), Some("Control+Shift+O"));
    }
}
