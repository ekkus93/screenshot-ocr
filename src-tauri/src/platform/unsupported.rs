use crate::cancellation::CancellationToken;
use crate::capture::CaptureBackend;
use crate::diagnostics::PlatformDiagnostics;
use crate::error::AppError;
use crate::models::CaptureBackendPreference;
use crate::ocr::OcrRuntime;
use crate::platform::PlatformServices;
use async_trait::async_trait;

#[derive(Debug, Default)]
pub struct UnsupportedPlatform;

#[async_trait]
impl PlatformServices for UnsupportedPlatform {
    async fn prepare_capture(
        &self,
        _preference: CaptureBackendPreference,
        cancellation: &CancellationToken,
    ) -> Result<Box<dyn CaptureBackend>, AppError> {
        cancellation.check()?;
        Err(AppError::UnsupportedEnvironment)
    }

    fn resolve_ocr_runtime(&self) -> Result<OcrRuntime, AppError> {
        Err(AppError::UnsupportedEnvironment)
    }

    async fn diagnostics(&self) -> Result<PlatformDiagnostics, AppError> {
        Err(AppError::UnsupportedEnvironment)
    }

    fn direct_shortcut(&self) -> Option<&'static str> {
        None
    }

    fn shortcut_unavailable_status(&self) -> &'static str {
        "unavailable on this platform"
    }
}
