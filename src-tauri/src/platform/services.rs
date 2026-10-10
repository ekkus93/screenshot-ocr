use crate::cancellation::CancellationToken;
use crate::capture::CaptureBackend;
use crate::diagnostics::PlatformDiagnostics;
use crate::error::AppError;
use crate::models::CaptureBackendPreference;
use crate::ocr::OcrRuntime;
use async_trait::async_trait;

#[async_trait]
pub trait PlatformServices: Send + Sync {
    async fn prepare_capture(
        &self,
        preference: CaptureBackendPreference,
        cancellation: &CancellationToken,
    ) -> Result<Box<dyn CaptureBackend>, AppError>;

    fn resolve_ocr_runtime(&self) -> Result<OcrRuntime, AppError>;

    async fn diagnostics(&self) -> Result<PlatformDiagnostics, AppError>;

    fn direct_shortcut(&self) -> Option<&'static str>;

    fn shortcut_unavailable_status(&self) -> &'static str {
        "not registered; use the documented platform shortcut"
    }
}
