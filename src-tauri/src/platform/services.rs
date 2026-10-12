use crate::cancellation::CancellationToken;
use crate::capture::CaptureBackend;
use crate::diagnostics::PlatformDiagnostics;
use crate::error::AppError;
use crate::models::CaptureBackendPreference;
use crate::ocr::OcrRuntime;
use async_trait::async_trait;
use std::path::Path;

#[async_trait]
pub trait PlatformServices: Send + Sync {
    async fn prepare_capture(
        &self,
        preference: CaptureBackendPreference,
        cancellation: &CancellationToken,
    ) -> Result<Box<dyn CaptureBackend>, AppError>;

    fn resolve_ocr_runtime(&self) -> Result<OcrRuntime, AppError>;

    // File privacy is an OS mechanism, not part of the portable settings model.
    fn protect_settings_directory(&self, path: &Path) -> Result<(), AppError>;
    fn protect_settings_file(&self, path: &Path) -> Result<(), AppError>;

    // Preflight the directory before creation, and validate it before reads.
    // Platform adapters may reject junctions/reparse points without leaking
    // OS-specific path handling into the shared settings implementation.
    fn validate_settings_create_directory(&self, _path: &Path) -> Result<(), AppError> {
        Ok(())
    }

    fn validate_settings_read_directory(&self, _path: &Path) -> Result<(), AppError> {
        Ok(())
    }

    async fn diagnostics(&self) -> Result<PlatformDiagnostics, AppError>;

    fn direct_shortcut(&self) -> Option<&'static str>;

    fn shortcut_unavailable_status(&self) -> &'static str {
        "not registered; use the documented platform shortcut"
    }
}
