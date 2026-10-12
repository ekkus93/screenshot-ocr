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
use std::os::windows::fs::MetadataExt;
use std::path::Path;

const REPARSE_POINT: u32 = 0x400;

fn settings_path_is_owned(path: &Path, directory: bool) -> bool {
    // A regular leaf under a junctioned parent still redirects writes
    // outside the intended settings tree. Inspect every ancestor without
    // following reparse points; relative paths cannot establish ownership.
    if !path.is_absolute() {
        return false;
    }
    path.ancestors().enumerate().all(|(depth, candidate)| {
        candidate.symlink_metadata().is_ok_and(|metadata| {
            let expected_type = if depth == 0 && !directory {
                metadata.is_file()
            } else {
                metadata.is_dir()
            };
            expected_type && metadata.file_attributes() & REPARSE_POINT == 0
        })
    })
}

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

    // Preserve inherited Windows app-config ACLs, but never accept a
    // reparse-pointed settings directory or temporary file for writes.
    fn protect_settings_directory(&self, path: &Path) -> Result<(), AppError> {
        if settings_path_is_owned(path, true) {
            Ok(())
        } else {
            Err(AppError::SettingsWriteFailed)
        }
    }

    fn protect_settings_file(&self, path: &Path) -> Result<(), AppError> {
        if settings_path_is_owned(path, false) {
            Ok(())
        } else {
            Err(AppError::SettingsWriteFailed)
        }
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

    #[test]
    fn windows_settings_protection_rejects_substituted_file_types() {
        use std::fs;
        let temp = tempfile::tempdir().expect("tempdir");
        let config = temp.path().join("config");
        fs::write(&config, b"fixture").expect("write file");
        assert!(matches!(
            WindowsPlatform.protect_settings_directory(&config),
            Err(AppError::SettingsWriteFailed)
        ));
        fs::remove_file(&config).expect("remove file");
        fs::create_dir(&config).expect("create directory");
        assert!(WindowsPlatform.protect_settings_directory(&config).is_ok());
        let settings = config.join("settings.tmp");
        fs::create_dir(&settings).expect("create substituted directory");
        assert!(matches!(
            WindowsPlatform.protect_settings_file(&settings),
            Err(AppError::SettingsWriteFailed)
        ));
        fs::remove_dir(&settings).expect("remove directory");
        fs::write(&settings, b"fixture").expect("write regular file");
        assert!(WindowsPlatform.protect_settings_file(&settings).is_ok());
    }

    #[test]
    fn windows_settings_protection_rejects_reparse_pointed_parent() {
        use std::fs;
        let temp = tempfile::tempdir().expect("tempdir");
        let actual = temp.path().join("actual");
        let config = actual.join("config");
        fs::create_dir_all(&config).expect("create config");
        let settings = config.join("settings.tmp");
        fs::write(&settings, b"synthetic").expect("write settings");
        assert!(WindowsPlatform.protect_settings_directory(&config).is_ok());
        assert!(WindowsPlatform.protect_settings_file(&settings).is_ok());

        let linked = temp.path().join("linked");
        // Some hosted Windows accounts cannot create directory symlinks.
        // The regular-ancestor assertions above still run in that case.
        if std::os::windows::fs::symlink_dir(&actual, &linked).is_ok() {
            assert!(matches!(
                WindowsPlatform.protect_settings_directory(&linked.join("config")),
                Err(AppError::SettingsWriteFailed)
            ));
            assert!(matches!(
                WindowsPlatform.protect_settings_file(&linked.join("config/settings.tmp")),
                Err(AppError::SettingsWriteFailed)
            ));
        }
        assert!(!settings_path_is_owned(Path::new("relative/settings.tmp"), false));
    }

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
