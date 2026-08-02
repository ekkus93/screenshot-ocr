use crate::cancellation::CancellationToken;
use crate::capture::CaptureBackend;
use crate::error::AppError;
use crate::image_pipeline::decode_captured_image;
use crate::models::{CaptureBackendId, CapturedImage};
use async_trait::async_trait;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;
use tokio::process::{Child, Command};
use tokio::time::sleep;
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

    async fn capture_into(
        &self,
        output: &Path,
        cancellation: &CancellationToken,
    ) -> Result<CapturedImage, AppError> {
        cancellation.check()?;
        let mut command = Command::new(&self.executable);
        command
            .arg("--area")
            .arg("--file")
            .arg(output)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .kill_on_drop(true);
        let mut child = command
            .spawn()
            .map_err(|_| AppError::CaptureProcessFailed)?;
        let status = tokio::select! {
            biased;
            _ = cancellation.cancelled() => {
                terminate_child(&mut child).await?;
                return Err(AppError::CaptureCancelled);
            }
            _ = sleep(CAPTURE_TIMEOUT) => {
                terminate_child(&mut child).await?;
                return Err(AppError::CaptureTimedOut);
            }
            result = child.wait() => result.map_err(|_| AppError::CaptureProcessFailed)?,
        };
        cancellation.check()?;
        if !status.success() {
            return if status.code() == Some(1) {
                Err(AppError::CaptureCancelled)
            } else {
                Err(AppError::CaptureProcessFailed)
            };
        }
        validate_capture_output(output)?;
        let bytes = fs::read(output).map_err(|_| AppError::CaptureImageInvalid)?;
        cancellation.check()?;
        decode_captured_image(&bytes, CaptureBackendId::GnomeScreenshot)
    }
}

#[async_trait]
impl CaptureBackend for GnomeScreenshotBackend {
    async fn capture_region(
        &self,
        cancellation: &CancellationToken,
    ) -> Result<CapturedImage, AppError> {
        let directory = create_capture_directory()?;
        let output = directory.join(format!("capture-{}.png", Uuid::new_v4()));
        let capture_result = self.capture_into(&output, cancellation).await;
        let cleanup_result = cleanup_directory(&directory);
        match (capture_result, cleanup_result) {
            (_, Err(error)) => Err(error),
            (result, Ok(())) => result,
        }
    }
}

async fn terminate_child(child: &mut Child) -> Result<(), AppError> {
    match child
        .try_wait()
        .map_err(|_| AppError::CaptureProcessFailed)?
    {
        Some(_) => Ok(()),
        None => {
            child
                .start_kill()
                .map_err(|_| AppError::CaptureProcessFailed)?;
            child
                .wait()
                .await
                .map_err(|_| AppError::CaptureProcessFailed)?;
            Ok(())
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
    use tokio::time::timeout;

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

    #[cfg(unix)]
    #[tokio::test]
    async fn cancellation_terminates_capture_helper() {
        use std::os::unix::fs::PermissionsExt;

        let directory = tempdir().expect("tempdir");
        let helper = directory.path().join("slow-helper");
        fs::write(&helper, "#!/bin/sh\nsleep 30\n").expect("write helper");
        fs::set_permissions(&helper, fs::Permissions::from_mode(0o700))
            .expect("make helper executable");
        let backend = GnomeScreenshotBackend::new(helper);
        let cancellation = CancellationToken::new();
        let task_token = cancellation.clone();
        let task = tokio::spawn(async move { backend.capture_region(&task_token).await });
        sleep(Duration::from_millis(50)).await;
        cancellation.cancel();
        let result = timeout(Duration::from_secs(2), task)
            .await
            .expect("capture cancellation timed out")
            .expect("capture task failed");
        assert!(matches!(result, Err(AppError::CaptureCancelled)));
    }
}
