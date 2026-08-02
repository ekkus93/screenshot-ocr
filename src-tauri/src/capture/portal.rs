use crate::cancellation::CancellationToken;
use crate::capture::CaptureBackend;
use crate::error::AppError;
use crate::image_pipeline::decode_captured_image;
use crate::models::{CaptureBackendId, CapturedImage};
use ashpd::desktop::{
    screenshot::{AvailableTargets, Screenshot, ScreenshotProxy},
    ResponseError,
};
use ashpd::{Error as PortalClientError, PortalError};
use async_trait::async_trait;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::time::timeout;
use url::Url;

const MINIMUM_AREA_VERSION: u32 = 3;
const PROBE_TIMEOUT: Duration = Duration::from_secs(5);
const CAPTURE_TIMEOUT: Duration = Duration::from_secs(120);
const MAX_CAPTURE_BYTES: u64 = 20 * 1024 * 1024;

#[derive(Clone, Debug, Default)]
pub struct PortalScreenshotBackend;

impl PortalScreenshotBackend {
    pub async fn probe_area_support(
        cancellation: &CancellationToken,
    ) -> Result<bool, AppError> {
        cancellation.check()?;
        let proxy = tokio::select! {
            biased;
            _ = cancellation.cancelled() => return Err(AppError::CaptureCancelled),
            result = timeout(PROBE_TIMEOUT, ScreenshotProxy::new()) => result
                .map_err(|_| AppError::CaptureBackendUnavailable)?
                .map_err(map_portal_error)?,
        };
        let version = proxy.version();
        if version < MINIMUM_AREA_VERSION {
            return Ok(false);
        }
        let targets = tokio::select! {
            biased;
            _ = cancellation.cancelled() => return Err(AppError::CaptureCancelled),
            result = timeout(PROBE_TIMEOUT, proxy.available_targets()) => result
                .map_err(|_| AppError::CaptureBackendUnavailable)?
                .map_err(map_portal_error)?,
        };
        Ok(supports_area(
            version,
            targets.contains(AvailableTargets::Area),
        ))
    }

    async fn capture_area(
        &self,
        cancellation: &CancellationToken,
    ) -> Result<CapturedImage, AppError> {
        cancellation.check()?;
        let request = tokio::select! {
            biased;
            _ = cancellation.cancelled() => return Err(AppError::CaptureCancelled),
            result = timeout(
                CAPTURE_TIMEOUT,
                Screenshot::request()
                    .interactive(true)
                    .modal(true)
                    .target(AvailableTargets::Area)
                    .send(),
            ) => result
                .map_err(|_| AppError::CaptureTimedOut)?
                .map_err(map_portal_error)?,
        };
        cancellation.check()?;
        let response = request.response().map_err(map_portal_error)?;
        cancellation.check()?;
        load_portal_result(response.uri().as_str())
    }
}

#[async_trait]
impl CaptureBackend for PortalScreenshotBackend {
    async fn capture_region(
        &self,
        cancellation: &CancellationToken,
    ) -> Result<CapturedImage, AppError> {
        self.capture_area(cancellation).await
    }
}

fn supports_area(version: u32, area_advertised: bool) -> bool {
    version >= MINIMUM_AREA_VERSION && area_advertised
}

fn map_portal_error(error: PortalClientError) -> AppError {
    match error {
        PortalClientError::Response(ResponseError::Cancelled)
        | PortalClientError::Portal(PortalError::Cancelled(_)) => AppError::CaptureCancelled,
        PortalClientError::Portal(PortalError::NotAllowed(_)) => AppError::CapturePermissionDenied,
        PortalClientError::RequiresVersion(_, _) | PortalClientError::PortalNotFound(_) => {
            AppError::CaptureBackendUnavailable
        }
        _ => AppError::CaptureProcessFailed,
    }
}

fn load_portal_result(uri: &str) -> Result<CapturedImage, AppError> {
    let path = file_path_from_uri(uri)?;
    validate_portal_file(&path)?;
    let bytes = read_bounded(&path)?;
    decode_captured_image(&bytes, CaptureBackendId::XdgPortal)
}

fn file_path_from_uri(uri: &str) -> Result<PathBuf, AppError> {
    let parsed = Url::parse(uri).map_err(|_| AppError::CaptureResultUriInvalid)?;
    if parsed.scheme() != "file" {
        return Err(AppError::CaptureResultUriInvalid);
    }
    let path = parsed
        .to_file_path()
        .map_err(|()| AppError::CaptureResultUriInvalid)?;
    if !path.is_absolute() {
        return Err(AppError::CaptureResultUriInvalid);
    }
    Ok(path)
}

fn validate_portal_file(path: &Path) -> Result<(), AppError> {
    let metadata = fs::symlink_metadata(path).map_err(|_| AppError::CaptureResultMissing)?;
    let file_type = metadata.file_type();
    if !file_type.is_file() || file_type.is_symlink() || metadata.len() == 0 {
        return Err(AppError::CaptureImageInvalid);
    }
    if metadata.len() > MAX_CAPTURE_BYTES {
        return Err(AppError::CaptureTooLarge);
    }
    Ok(())
}

fn read_bounded(path: &Path) -> Result<Vec<u8>, AppError> {
    let file = fs::File::open(path).map_err(|_| AppError::CaptureResultMissing)?;
    let mut bytes = Vec::new();
    file.take(MAX_CAPTURE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| AppError::CaptureImageInvalid)?;
    if bytes.len() as u64 > MAX_CAPTURE_BYTES {
        return Err(AppError::CaptureTooLarge);
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn area_requires_version_three_and_advertisement() {
        assert!(!supports_area(2, true));
        assert!(!supports_area(3, false));
        assert!(supports_area(3, true));
    }

    #[tokio::test]
    async fn pre_cancelled_probe_fails_without_portal_access() {
        let cancellation = CancellationToken::new();
        cancellation.cancel();
        assert!(matches!(
            PortalScreenshotBackend::probe_area_support(&cancellation).await,
            Err(AppError::CaptureCancelled)
        ));
    }

    #[test]
    fn accepts_only_absolute_file_uris() {
        assert_eq!(
            file_path_from_uri("file:///tmp/capture.png").expect("file URI"),
            PathBuf::from("/tmp/capture.png")
        );
        assert!(matches!(
            file_path_from_uri("https://example.invalid/capture.png"),
            Err(AppError::CaptureResultUriInvalid)
        ));
        assert!(matches!(
            file_path_from_uri("not a uri"),
            Err(AppError::CaptureResultUriInvalid)
        ));
    }

    #[test]
    fn bounded_reader_rejects_oversized_files() {
        let directory = tempdir().expect("tempdir");
        let path = directory.path().join("capture.png");
        let file = fs::File::create(&path).expect("create capture");
        file.set_len(MAX_CAPTURE_BYTES + 1).expect("extend capture");
        assert!(matches!(
            validate_portal_file(&path),
            Err(AppError::CaptureTooLarge)
        ));
    }

    #[cfg(unix)]
    #[test]
    fn portal_result_rejects_symlinks() {
        use std::os::unix::fs::symlink;

        let directory = tempdir().expect("tempdir");
        let target = directory.path().join("target.png");
        let link = directory.path().join("capture.png");
        fs::write(&target, b"pixels").expect("write target");
        symlink(&target, &link).expect("create symlink");
        assert!(matches!(
            validate_portal_file(&link),
            Err(AppError::CaptureImageInvalid)
        ));
    }
}
