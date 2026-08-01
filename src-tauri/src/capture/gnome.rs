use crate::capture::CaptureBackend;
use crate::error::AppError;
use crate::image_pipeline::decode_captured_image;
use crate::models::{CaptureBackendId, CapturedImage};
use async_trait::async_trait;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Stdio;
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

    async fn capture_into(&self, output: &Path) -> Result<CapturedImage, AppError> {
        let mut command = Command::new(&self.executable);
        command
            .arg("--area")
            .arg("--file")
            .arg(output)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .kill_on_drop(true);
        let status = timeout(CAPTURE_TIMEOUT, command.status())
            .await
            .map_err(|_| AppError::CaptureProcessFailed)?
            .map_err(|_| AppError::CaptureProcessFailed)?;
        if !status.success() {
            return if status.code() == Some(1) {
                Err(AppError::CaptureCancelled)
            } else {
                Err(AppError::CaptureProcessFailed)
            };
        }
        validate_capture_output(output)?;
        let bytes = fs::read(output).map_err(|_| AppError::CaptureImageInvalid)?;
        decode_captured_image(&bytes, CaptureBackendId::GnomeScreenshot)
    }
}

#[async_trait]
impl CaptureBackend for GnomeScreenshotBackend {
    async fn capture_region(&self) -> Result<CapturedImage, AppError> {
        let directory = create_capture_directory()?;
        let output = directory.join(format!("capture-{}.png", Uuid::new_v4()));
        let capture_result = self.capture_into(&output).await;
        let cleanup_result = cleanup_directory(&directory);
        match (capture_result, cleanup_result) {
            (_, Err(error)) => Err(error),
            (result, Ok(())) => result,
        }
    }
}

fn validate_capture_output(path: &Path) -> Result<(), AppError> {
    let metadata = fs::symlink_metadata(path).map_err(|_| AppError::CaptureResultMissing)?;
    if metadata.file_type().is_file() && !metadata.file_type().is_symlink() && metadata.len() > 0 {
        Ok(())
    } else {
        Err(AppError::CaptureImageInvalid)
    }
}

fn create_capture_directory() -> Result<PathBuf, AppError> {
    let runtime_directory = std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute() && path.is_dir());
    let base = runtime_directory.unwrap_or_else(std::env::temp_dir);
    let directory = base.join(format!("screenshot-ocr-{}", Uuid::new_v4()));
    fs::create_dir(&directory).map_err(|_| AppError::CaptureProcessFailed)?;
    set_private_permissions(&directory)?;
    Ok(directory)
}

#[cfg(unix)]
fn set_private_permissions(path: &Path) -> Result<(), AppError> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
        .map_err(|_| AppError::CaptureProcessFailed)
}

#[cfg(not(unix))]
fn set_private_permissions(_path: &Path) -> Result<(), AppError> {
    Ok(())
}

fn cleanup_directory(directory: &Path) -> Result<(), AppError> {
    fs::remove_dir_all(directory).map_err(|_| AppError::TemporaryCleanupFailed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn cleanup_removes_nested_capture_artifacts() {
        let parent = tempdir().expect("tempdir");
        let directory = parent.path().join("capture");
        fs::create_dir(&directory).expect("create capture directory");
        fs::write(directory.join("capture.png"), b"private pixels").expect("write capture");
        cleanup_directory(&directory).expect("cleanup capture directory");
        assert!(!directory.exists());
    }

    #[cfg(unix)]
    #[test]
    fn output_validation_rejects_symlinks() {
        use std::os::unix::fs::symlink;

        let directory = tempdir().expect("tempdir");
        let target = directory.path().join("target.png");
        let link = directory.path().join("capture.png");
        fs::write(&target, b"not an image").expect("write target");
        symlink(&target, &link).expect("create symlink");
        assert!(matches!(
            validate_capture_output(&link),
            Err(AppError::CaptureImageInvalid)
        ));
    }
}
