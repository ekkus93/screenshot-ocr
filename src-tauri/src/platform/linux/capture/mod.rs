mod gnome;
mod image;
mod portal;

use super::environment::LinuxEnvironmentInfo;
use crate::cancellation::CancellationToken;
use crate::capture::CaptureBackend;
use crate::error::AppError;
use crate::models::{CaptureBackendId, CaptureBackendPreference};

pub use gnome::GnomeScreenshotBackend;
pub use portal::PortalScreenshotBackend;

pub async fn prepare_capture_backend(
    preference: CaptureBackendPreference,
    environment: &LinuxEnvironmentInfo,
    cancellation: &CancellationToken,
) -> Result<Box<dyn CaptureBackend>, AppError> {
    cancellation.check()?;
    let portal_supported = match preference {
        CaptureBackendPreference::Gnome => false,
        CaptureBackendPreference::Auto => {
            match PortalScreenshotBackend::probe_area_support(cancellation).await {
                Ok(supported) => supported,
                Err(AppError::CaptureCancelled) => return Err(AppError::CaptureCancelled),
                Err(_) => false,
            }
        }
        CaptureBackendPreference::Portal => {
            PortalScreenshotBackend::probe_area_support(cancellation).await?
        }
    };
    cancellation.check()?;
    match choose_backend_id(
        preference,
        portal_supported,
        environment.gnome_screenshot.is_some(),
    )? {
        CaptureBackendId::XdgPortal => Ok(Box::new(PortalScreenshotBackend)),
        CaptureBackendId::GnomeScreenshot => environment
            .gnome_screenshot
            .clone()
            .map(GnomeScreenshotBackend::new)
            .map(|backend| Box::new(backend) as Box<dyn CaptureBackend>)
            .ok_or(AppError::CaptureBackendUnavailable),
        CaptureBackendId::WindowsNative => Err(AppError::CaptureBackendUnavailable),
    }
}

fn choose_backend_id(
    preference: CaptureBackendPreference,
    portal_supported: bool,
    gnome_available: bool,
) -> Result<CaptureBackendId, AppError> {
    match preference {
        CaptureBackendPreference::Portal if portal_supported => Ok(CaptureBackendId::XdgPortal),
        CaptureBackendPreference::Portal => Err(AppError::CaptureBackendUnavailable),
        CaptureBackendPreference::Gnome if gnome_available => Ok(CaptureBackendId::GnomeScreenshot),
        CaptureBackendPreference::Gnome => Err(AppError::CaptureBackendUnavailable),
        CaptureBackendPreference::Auto if portal_supported => Ok(CaptureBackendId::XdgPortal),
        CaptureBackendPreference::Auto if gnome_available => Ok(CaptureBackendId::GnomeScreenshot),
        CaptureBackendPreference::Auto => Err(AppError::CaptureBackendUnavailable),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn automatic_selection_prefers_proven_portal_area_capture() {
        assert_eq!(
            choose_backend_id(CaptureBackendPreference::Auto, true, true).expect("portal backend"),
            CaptureBackendId::XdgPortal
        );
    }

    #[test]
    fn automatic_selection_falls_back_before_opening_a_selector() {
        assert_eq!(
            choose_backend_id(CaptureBackendPreference::Auto, false, true).expect("GNOME backend"),
            CaptureBackendId::GnomeScreenshot
        );
    }

    #[test]
    fn explicit_portal_selection_fails_closed_without_area_support() {
        assert!(matches!(
            choose_backend_id(CaptureBackendPreference::Portal, false, true),
            Err(AppError::CaptureBackendUnavailable)
        ));
    }

    #[test]
    fn automatic_selection_rejects_missing_backends() {
        assert!(matches!(
            choose_backend_id(CaptureBackendPreference::Auto, false, false),
            Err(AppError::CaptureBackendUnavailable)
        ));
    }
}
