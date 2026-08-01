use crate::error::AppError;
use crate::image_pipeline::decode_captured_image;
use crate::models::{CaptureBackendId, CapturedImage};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;
use uuid::Uuid;

const CAPTURE_TIMEOUT: Duration = Duration::from_secs(120);

#[derive(Clone, Debug)]
pub struct GnomeScreenshotBackend {
    executable: PathBuf,
}

impl GnomeScreenshotBackend {
    pub fn new(executable: PathBuf) -> Self {
        Self { executable }
    }

    pub async fn capture_region(&self) -> Result<CapturedImage, AppError> {
        let directory = create_capture_directory()?;
        let output = directory.join(format!("capture-{}.png", Uuid::new_v4()));
        let mut command = Command::new(&self.executable);
        command.arg("--area").arg("--file").arg(&output).kill_on_drop(true);
        let status = timeout(CAPTURE_TIMEOUT, command.status()).await.map_err(|_| AppError::CaptureProcessFailed)?.map_err(|_| AppError::CaptureProcessFailed)?;
        if !status.success() {
            cleanup_directory(&directory);
            return if status.code() == Some(1) { Err(AppError::CaptureCancelled) } else { Err(AppError::CaptureProcessFailed) };
        }
        let metadata = fs::symlink_metadata(&output).map_err(|_| AppError::CaptureResultMissing)?;
        if !metadata.file_type().is_file() || metadata.file_type().is_symlink() || metadata.len() == 0 {
            cleanup_directory(&directory);
            return Err(AppError::CaptureImageInvalid);
        }
        let bytes = fs::read(&output).map_err(|_| AppError::CaptureImageInvalid)?;
        fs::remove_file(&output).map_err(|_| AppError::TemporaryCleanupFailed)?;
        let decoded = decode_captured_image(&bytes, CaptureBackendId::GnomeScreenshot)?;
        cleanup_directory(&directory);
        Ok(decoded)
    }
}

fn create_capture_directory() -> Result<PathBuf, AppError> {
    let base = std::env::var_os("XDG_RUNTIME_DIR").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
    let directory = base.join(format!("screenshot-ocr-{}", Uuid::new_v4()));
    fs::create_dir(&directory).map_err(|_| AppError::CaptureProcessFailed)?;
    set_private_permissions(&directory)?;
    Ok(directory)
}

#[cfg(unix)]
fn set_private_permissions(path: &Path) -> Result<(), AppError> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).map_err(|_| AppError::CaptureProcessFailed)
}

#[cfg(not(unix))]
fn set_private_permissions(_path: &Path) -> Result<(), AppError> { Ok(()) }

fn cleanup_directory(directory: &Path) {
    let _ = fs::remove_dir_all(directory);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capture_timeout_is_bounded() {
        assert!(CAPTURE_TIMEOUT <= Duration::from_secs(120));
    }
}
