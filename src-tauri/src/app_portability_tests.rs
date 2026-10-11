//! Platform-contract tests exercise the shared orchestration without a real desktop.
//! Fake services deliberately model Linux and Windows captures identically.

use super::*;
use crate::capture::CaptureBackend;
use crate::diagnostics::PlatformDiagnostics;
use crate::models::{
    CaptureBackendId, CaptureBackendPreference, CaptureSource, CapturedImage, CopyPolicy, TextMode,
};
use async_trait::async_trait;
use image::DynamicImage;
use std::sync::{Arc, Mutex};
use tempfile::tempdir;

#[derive(Clone, Copy)]
enum FakeOutcome {
    PrepareUnavailable,
    CaptureUnavailable,
    CancelDuringCapture,
    Captured,
}

type Events = Arc<Mutex<Vec<&'static str>>>;

fn note(events: &Events, event: &'static str) {
    events.lock().expect("events lock").push(event);
}

fn recorded(events: &Events) -> Vec<&'static str> {
    events.lock().expect("events lock").clone()
}

struct FakePlatform {
    outcome: FakeOutcome,
    backend: CaptureBackendId,
    events: Events,
}

struct FakeCapture {
    outcome: FakeOutcome,
    backend: CaptureBackendId,
    events: Events,
}

#[async_trait]
impl CaptureBackend for FakeCapture {
    async fn capture_region(
        &self,
        cancellation: &CancellationToken,
    ) -> Result<CapturedImage, AppError> {
        note(&self.events, "capture");
        cancellation.check()?;
        match self.outcome {
            FakeOutcome::CaptureUnavailable => Err(AppError::CaptureBackendUnavailable),
            FakeOutcome::CancelDuringCapture => {
                cancellation.cancel();
                Err(AppError::CaptureCancelled)
            }
            FakeOutcome::Captured => Ok(CapturedImage {
                image: DynamicImage::new_rgb8(4, 4),
                backend: self.backend,
                cleanup_failed: false,
            }),
            FakeOutcome::PrepareUnavailable => unreachable!("preparation rejects this mode"),
        }
    }
}

#[async_trait]
impl PlatformServices for FakePlatform {
    async fn prepare_capture(
        &self,
        _preference: CaptureBackendPreference,
        cancellation: &CancellationToken,
    ) -> Result<Box<dyn CaptureBackend>, AppError> {
        note(&self.events, "prepare");
        cancellation.check()?;
        if matches!(self.outcome, FakeOutcome::PrepareUnavailable) {
            return Err(AppError::CaptureBackendUnavailable);
        }
        Ok(Box::new(FakeCapture {
            outcome: self.outcome,
            backend: self.backend,
            events: self.events.clone(),
        }))
    }

    fn resolve_ocr_runtime(&self) -> Result<crate::ocr::OcrRuntime, AppError> {
        note(&self.events, "runtime");
        Err(AppError::OcrEngineUnavailable)
    }

    async fn diagnostics(&self) -> Result<PlatformDiagnostics, AppError> {
        Ok(PlatformDiagnostics {
            platform: "test".into(),
            os_release: "test".into(),
            desktop_environment: "test".into(),
            session_type: "test".into(),
            capture_summary: "available".into(),
            ocr_runtime: "unavailable".into(),
        })
    }

    fn direct_shortcut(&self) -> Option<&'static str> {
        None
    }
}

fn request() -> CaptureRequest {
    CaptureRequest {
        job_id: CaptureJobId::new(),
        mode: TextMode::Terminal,
        language: "eng".into(),
        copy_policy: CopyPolicy::Preview,
        source: CaptureSource::MainWindow,
    }
}

async fn run_case(
    outcome: FakeOutcome,
    backend: CaptureBackendId,
) -> (
    Result<OcrResult, AppError>,
    Events,
    CaptureJobId,
    AppServices,
) {
    let events: Events = Arc::new(Mutex::new(Vec::new()));
    let directory = tempdir().expect("tempdir");
    let services = AppServices::with_platform(
        directory.path().to_path_buf(),
        Arc::new(FakePlatform {
            outcome,
            backend,
            events: events.clone(),
        }),
    );
    let request = request();
    let job_id = request.job_id;
    let before = events.clone();
    let after = events.clone();
    let result = services
        .capture(
            request,
            move || {
                note(&before, "hide");
                Ok(())
            },
            move || {
                note(&after, "restore");
                Ok(())
            },
        )
        .await;
    (result, events, job_id, services)
}

#[tokio::test]
async fn failed_preparation_never_hides_window() {
    let (result, events, _, _) = run_case(
        FakeOutcome::PrepareUnavailable,
        CaptureBackendId::GnomeScreenshot,
    )
    .await;
    assert!(matches!(result, Err(AppError::CaptureBackendUnavailable)));
    assert_eq!(recorded(&events), ["prepare"]);
}

#[tokio::test]
async fn failed_region_capture_restores_window_before_returning() {
    let (result, events, _, _) =
        run_case(FakeOutcome::CaptureUnavailable, CaptureBackendId::XdgPortal).await;
    assert!(matches!(result, Err(AppError::CaptureBackendUnavailable)));
    assert_eq!(recorded(&events), ["prepare", "hide", "capture", "restore"]);
}

#[tokio::test]
async fn selection_cancellation_restores_window_and_retains_job_ownership() {
    let (result, events, job_id, services) = run_case(
        FakeOutcome::CancelDuringCapture,
        CaptureBackendId::WindowsNative,
    )
    .await;
    assert!(matches!(result, Err(AppError::CaptureCancelled)));
    assert_eq!(recorded(&events), ["prepare", "hide", "capture", "restore"]);
    let mut state = services.state.lock().await;
    assert_eq!(state.active_job_id(), Some(job_id));
    assert!(matches!(
        state.begin(CaptureJobId::new()),
        Err(AppError::CaptureAlreadyActive)
    ));
    state.finish(job_id);
    assert!(state.begin(CaptureJobId::new()).is_ok());
}

#[tokio::test]
async fn successful_platform_capture_restores_window_before_ocr_runtime_resolution() {
    for backend in [
        CaptureBackendId::GnomeScreenshot,
        CaptureBackendId::XdgPortal,
        CaptureBackendId::WindowsNative,
    ] {
        let (result, events, _, _) = run_case(FakeOutcome::Captured, backend).await;
        assert!(matches!(result, Err(AppError::OcrEngineUnavailable)));
        assert_eq!(
            recorded(&events),
            ["prepare", "hide", "capture", "restore", "runtime"]
        );
    }
}

#[tokio::test]
async fn invalid_request_never_calls_platform_services() {
    let events: Events = Arc::new(Mutex::new(Vec::new()));
    let directory = tempdir().expect("tempdir");
    let services = AppServices::with_platform(
        directory.path().to_path_buf(),
        Arc::new(FakePlatform {
            outcome: FakeOutcome::Captured,
            backend: CaptureBackendId::WindowsNative,
            events: events.clone(),
        }),
    );
    let mut invalid = request();
    invalid.language = "not-supported".into();
    let result = services.capture(invalid, || Ok(()), || Ok(())).await;
    assert!(matches!(result, Err(AppError::SettingsInvalid)));
    assert!(recorded(&events).is_empty());
}

#[cfg(unix)]
struct SuccessfulFakePlatform {
    executable: std::path::PathBuf,
    events: Events,
}

#[cfg(unix)]
struct SuccessfulFakeCapture {
    events: Events,
}

#[cfg(unix)]
#[async_trait]
impl CaptureBackend for SuccessfulFakeCapture {
    async fn capture_region(
        &self,
        cancellation: &CancellationToken,
    ) -> Result<CapturedImage, AppError> {
        note(&self.events, "capture");
        cancellation.check()?;
        Ok(CapturedImage {
            image: DynamicImage::new_rgb8(4, 4),
            backend: CaptureBackendId::WindowsNative,
            cleanup_failed: true,
        })
    }
}

#[cfg(unix)]
#[async_trait]
impl PlatformServices for SuccessfulFakePlatform {
    async fn prepare_capture(
        &self,
        _preference: CaptureBackendPreference,
        cancellation: &CancellationToken,
    ) -> Result<Box<dyn CaptureBackend>, AppError> {
        note(&self.events, "prepare");
        cancellation.check()?;
        Ok(Box::new(SuccessfulFakeCapture {
            events: self.events.clone(),
        }))
    }

    fn resolve_ocr_runtime(&self) -> Result<crate::ocr::OcrRuntime, AppError> {
        note(&self.events, "runtime");
        Ok(crate::ocr::OcrRuntime::new(self.executable.clone(), None))
    }

    async fn diagnostics(&self) -> Result<PlatformDiagnostics, AppError> {
        Ok(PlatformDiagnostics {
            platform: "test".into(),
            os_release: "test".into(),
            desktop_environment: "test".into(),
            session_type: "test".into(),
            capture_summary: "available".into(),
            ocr_runtime: "available".into(),
        })
    }

    fn direct_shortcut(&self) -> Option<&'static str> {
        None
    }
}

#[cfg(unix)]
fn write_successful_fake_tesseract(path: &std::path::Path) {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    fs::write(
        path,
        "#!/bin/sh\nif [ \"$1\" = \"--list-langs\" ]; then printf 'eng\\n'; exit 0; fi\ncat >/dev/null\nprintf 'portable text\\n'\n",
    )
    .expect("write fake tesseract");
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
        .expect("make fake tesseract executable");
}

#[cfg(unix)]
#[tokio::test]
async fn successful_fake_capture_propagates_cleanup_warning() {
    let directory = tempdir().expect("tempdir");
    let executable = directory.path().join("fake-tesseract");
    write_successful_fake_tesseract(&executable);
    let events: Events = Arc::new(Mutex::new(Vec::new()));
    let services = AppServices::with_platform(
        directory.path().to_path_buf(),
        Arc::new(SuccessfulFakePlatform {
            executable,
            events: events.clone(),
        }),
    );

    let result = services
        .capture(
            request(),
            {
                let events = events.clone();
                move || {
                    note(&events, "hide");
                    Ok(())
                }
            },
            {
                let events = events.clone();
                move || {
                    note(&events, "restore");
                    Ok(())
                }
            },
        )
        .await
        .expect("successful portable capture");

    assert_eq!(result.backend, CaptureBackendId::WindowsNative);
    assert_eq!(result.text.trim(), "portable text");
    assert!(result
        .warnings
        .iter()
        .any(|warning| warning.code == "temporary_cleanup_failed"));
    assert_eq!(
        services
            .runtime_diagnostics
            .snapshot()
            .cleanup_failure_count,
        1
    );
    assert_eq!(
        recorded(&events),
        ["prepare", "hide", "capture", "restore", "runtime"]
    );
}

#[cfg(unix)]
#[tokio::test]
async fn missing_english_language_propagates_through_shared_capture() {
    let directory = tempdir().expect("tempdir");
    let executable = directory.path().join("fake-tesseract");
    write_executable(
        &executable,
        "#!/bin/sh\nif [ \"$1\" = \"--list-langs\" ]; then printf 'osd\\n'; exit 0; fi\nexit 1\n",
    );
    let events: Events = Arc::new(Mutex::new(Vec::new()));
    let services = AppServices::with_platform(
        directory.path().to_path_buf(),
        Arc::new(SuccessfulFakePlatform {
            executable,
            events: events.clone(),
        }),
    );
    let result = services
        .capture(request(), || Ok(()), || Ok(()))
        .await;
    assert!(matches!(result, Err(AppError::OcrLanguageMissing)));
    assert_eq!(recorded(&events), ["prepare", "capture", "runtime"]);
}

#[cfg(unix)]
#[tokio::test]
async fn cancelling_during_shared_ocr_preserves_job_ownership() {
    use std::time::Duration;
    use tokio::time::{sleep, timeout};

    let directory = tempdir().expect("tempdir");
    let executable = directory.path().join("slow-tesseract");
    let marker = directory.path().join("recognize-started");
    write_executable(
        &executable,
        &format!(
            "#!/bin/sh\nif [ \"$1\" = \"--list-langs\" ]; then printf 'eng\\n'; exit 0; fi\ncat >/dev/null\ntouch '{}'\nsleep 30\n",
            marker.display()
        ),
    );
    let events: Events = Arc::new(Mutex::new(Vec::new()));
    let services = AppServices::with_platform(
        directory.path().to_path_buf(),
        Arc::new(SuccessfulFakePlatform {
            executable,
            events: events.clone(),
        }),
    );
    let request = request();
    let job_id = request.job_id;
    let capture_services = services.clone();
    let capture = tokio::spawn(async move {
        capture_services.capture(request, || Ok(()), || Ok(())).await
    });

    timeout(Duration::from_secs(8), async {
        while !marker.is_file() {
            sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .expect("fake OCR child did not start");

    {
        let state = services.state.lock().await;
        state.cancel(job_id).expect("cancel active capture");
        assert_eq!(state.active_job_id(), Some(job_id));
    }
    let result = timeout(Duration::from_secs(4), capture)
        .await
        .expect("shared OCR cancellation timed out")
        .expect("capture task panicked");
    assert!(matches!(result, Err(AppError::CaptureCancelled)));
    assert_eq!(recorded(&events), ["prepare", "capture", "runtime"]);
    let mut state = services.state.lock().await;
    assert_eq!(state.active_job_id(), Some(job_id));
    assert!(matches!(
        state.begin(CaptureJobId::new()),
        Err(AppError::CaptureAlreadyActive)
    ));
    state.finish(job_id);
    assert!(state.begin(CaptureJobId::new()).is_ok());
}
