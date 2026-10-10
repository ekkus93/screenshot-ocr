mod capture;
mod environment;

use crate::cancellation::CancellationToken;
use crate::capture::CaptureBackend;
use crate::diagnostics::PlatformDiagnostics;
use crate::error::AppError;
use crate::models::CaptureBackendPreference;
use crate::ocr::OcrRuntime;
use crate::platform::PlatformServices;
use async_trait::async_trait;
use capture::{prepare_capture_backend, PortalScreenshotBackend};
use environment::{LinuxEnvironmentInfo, LinuxEnvironmentProbe};

#[derive(Debug, Default)]
pub struct LinuxPlatform;

#[async_trait]
impl PlatformServices for LinuxPlatform {
    async fn prepare_capture(
        &self,
        preference: CaptureBackendPreference,
        cancellation: &CancellationToken,
    ) -> Result<Box<dyn CaptureBackend>, AppError> {
        let environment = LinuxEnvironmentProbe::probe();
        prepare_capture_backend(preference, &environment, cancellation).await
    }

    fn resolve_ocr_runtime(&self) -> Result<OcrRuntime, AppError> {
        let environment = LinuxEnvironmentProbe::probe();
        environment
            .tesseract
            .map(|executable| OcrRuntime::new(executable, None))
            .ok_or(AppError::OcrEngineUnavailable)
    }

    async fn diagnostics(&self) -> Result<PlatformDiagnostics, AppError> {
        let environment = LinuxEnvironmentProbe::probe();
        let portal_summary = PortalScreenshotBackend::safe_capability_summary().await;
        Ok(platform_diagnostics(&environment, portal_summary))
    }

    fn direct_shortcut(&self) -> Option<&'static str> {
        direct_shortcut_for_session(std::env::var("XDG_SESSION_TYPE").ok().as_deref())
    }

    fn shortcut_unavailable_status(&self) -> &'static str {
        "deferred; use the GNOME custom shortcut"
    }
}

fn platform_diagnostics(
    environment: &LinuxEnvironmentInfo,
    portal_summary: String,
) -> PlatformDiagnostics {
    PlatformDiagnostics {
        platform: "linux".into(),
        os_release: environment.os_release.clone(),
        desktop_environment: environment.desktop_environment.clone(),
        session_type: environment.session_type.clone(),
        capture_summary: format!(
            "portal: {portal_summary}; GNOME helper: {}",
            availability(environment.gnome_screenshot.is_some())
        ),
        ocr_runtime: availability(environment.tesseract.is_some()).into(),
    }
}

fn availability(available: bool) -> &'static str {
    if available {
        "available"
    } else {
        "unavailable"
    }
}

fn direct_shortcut_for_session(session: Option<&str>) -> Option<&'static str> {
    match session {
        Some(session) if session.eq_ignore_ascii_case("x11") => Some("Super+Shift+O"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn linux_direct_shortcut_is_x11_only() {
        assert_eq!(
            direct_shortcut_for_session(Some("x11")),
            Some("Super+Shift+O")
        );
        assert_eq!(direct_shortcut_for_session(Some("wayland")), None);
        assert_eq!(direct_shortcut_for_session(None), None);
    }

    #[test]
    fn diagnostics_never_include_executable_paths() {
        let environment = LinuxEnvironmentInfo {
            os_release: "Ubuntu".into(),
            desktop_environment: "GNOME".into(),
            session_type: "wayland".into(),
            gnome_screenshot: Some(std::path::PathBuf::from("/sensitive/gnome-screenshot")),
            tesseract: Some(PathBuf::from("/sensitive/tesseract")),
        };
        let diagnostics =
            platform_diagnostics(&environment, "Screenshot v3; area target available".into());
        let text = format!("{diagnostics:?}");
        assert!(!text.contains("/sensitive/"));
        assert_eq!(diagnostics.ocr_runtime, "available");
    }
}
