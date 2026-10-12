use crate::cancellation::CancellationToken;
use crate::error::AppError;
use crate::image_pipeline::{encode_png, PreparedVariant};
use crate::models::{OcrCandidate, OcrWarning, TextMode};
use crate::ocr::cleanup::{cleanup_text, score_text};
use crate::ocr::OcrRuntime;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::{Child, Command};
use tokio::time::{sleep_until, Instant};

const OCR_TIMEOUT: Duration = Duration::from_secs(30);
const LANGUAGE_PROBE_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_OCR_TEXT_BYTES: usize = 1_000_000;
const MAX_LANGUAGE_LIST_BYTES: usize = 64 * 1024;

#[derive(Clone, Debug)]
pub struct TesseractEngine {
    runtime: OcrRuntime,
}

impl TesseractEngine {
    pub fn new(runtime: OcrRuntime) -> Self {
        Self { runtime }
    }

    fn command(&self) -> Command {
        let mut command = Command::new(&self.runtime.executable);
        if let Some(directory) = &self.runtime.working_directory {
            command.current_dir(directory);
        }
        if let Some(tessdata_dir) = &self.runtime.tessdata_dir {
            command.env("TESSDATA_PREFIX", tessdata_dir);
        }
        command
    }

    pub async fn probe_english(
        &self,
        cancellation: &CancellationToken,
    ) -> Result<Vec<String>, AppError> {
        cancellation.check()?;
        let deadline = Instant::now() + LANGUAGE_PROBE_TIMEOUT;
        let mut child = self
            .command()
            .arg("--list-langs")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .map_err(|_| AppError::OcrEngineUnavailable)?;
        let stdout = match child.stdout.take() {
            Some(stdout) => stdout,
            None => {
                terminate_child(&mut child)
                    .await
                    .map_err(|_| AppError::OcrEngineUnavailable)?;
                return Err(AppError::OcrEngineUnavailable);
            }
        };
        let mut reader = tokio::spawn(async move {
            let mut limited = stdout.take((MAX_LANGUAGE_LIST_BYTES + 1) as u64);
            let mut bytes = Vec::new();
            limited
                .read_to_end(&mut bytes)
                .await
                .map_err(|_| AppError::OcrEngineUnavailable)?;
            Ok::<Vec<u8>, AppError>(bytes)
        });

        let status_result = tokio::select! {
            biased;
            _ = cancellation.cancelled() => {
                reader.abort();
                terminate_child(&mut child)
                    .await
                    .map_err(|_| AppError::OcrEngineUnavailable)?;
                return Err(AppError::CaptureCancelled);
            }
            _ = sleep_until(deadline) => {
                reader.abort();
                terminate_child(&mut child)
                    .await
                    .map_err(|_| AppError::OcrEngineUnavailable)?;
                return Err(AppError::OcrEngineUnavailable);
            }
            result = child.wait() => result.map_err(|_| AppError::OcrEngineUnavailable),
        };
        let status = status_result?;
        let output = tokio::select! {
            biased;
            _ = cancellation.cancelled() => {
                reader.abort();
                return Err(AppError::CaptureCancelled);
            }
            _ = sleep_until(deadline) => {
                reader.abort();
                return Err(AppError::OcrEngineUnavailable);
            }
            result = &mut reader => result
                .map_err(|_| AppError::OcrEngineUnavailable)??,
        };
        if !status.success() || output.len() > MAX_LANGUAGE_LIST_BYTES {
            return Err(AppError::OcrEngineUnavailable);
        }
        parse_languages(&output)
    }

    pub async fn recognize(
        &self,
        variant: &PreparedVariant,
        mode: TextMode,
        cancellation: &CancellationToken,
    ) -> Result<OcrCandidate, AppError> {
        cancellation.check()?;
        let deadline = Instant::now() + OCR_TIMEOUT;
        let png = encode_png(&variant.image)?;
        cancellation.check()?;
        let page_segmentation = match mode {
            TextMode::Terminal => "6",
            TextMode::Document => "3",
            TextMode::SingleLine => "7",
        };
        let mut child = self
            .command()
            .args([
                "stdin",
                "stdout",
                "-l",
                "eng",
                "--psm",
                page_segmentation,
                "-c",
                "preserve_interword_spaces=1",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .map_err(|_| AppError::OcrEngineUnavailable)?;
        let mut stdin = match child.stdin.take() {
            Some(stdin) => stdin,
            None => {
                terminate_child(&mut child).await?;
                return Err(AppError::OcrFailed);
            }
        };
        let write_result = tokio::select! {
            biased;
            _ = cancellation.cancelled() => Err(AppError::CaptureCancelled),
            _ = sleep_until(deadline) => Err(AppError::OcrTimedOut),
            result = stdin.write_all(&png) => result.map_err(|_| AppError::OcrFailed),
        };
        drop(stdin);
        if let Err(error) = write_result {
            terminate_child(&mut child).await?;
            return Err(error);
        }

        let stdout = match child.stdout.take() {
            Some(stdout) => stdout,
            None => {
                terminate_child(&mut child).await?;
                return Err(AppError::OcrFailed);
            }
        };
        let mut reader = tokio::spawn(async move {
            let mut limited = stdout.take((MAX_OCR_TEXT_BYTES + 1) as u64);
            let mut bytes = Vec::new();
            limited
                .read_to_end(&mut bytes)
                .await
                .map_err(|_| AppError::OcrFailed)?;
            Ok::<Vec<u8>, AppError>(bytes)
        });

        let status_result = tokio::select! {
            biased;
            _ = cancellation.cancelled() => {
                reader.abort();
                terminate_child(&mut child).await?;
                return Err(AppError::CaptureCancelled);
            }
            _ = sleep_until(deadline) => {
                reader.abort();
                terminate_child(&mut child).await?;
                return Err(AppError::OcrTimedOut);
            }
            result = child.wait() => result.map_err(|_| AppError::OcrFailed),
        };
        let status = match status_result {
            Ok(status) => status,
            Err(error) => {
                reader.abort();
                return Err(error);
            }
        };
        let output = tokio::select! {
            biased;
            _ = cancellation.cancelled() => {
                reader.abort();
                return Err(AppError::CaptureCancelled);
            }
            _ = sleep_until(deadline) => {
                reader.abort();
                return Err(AppError::OcrTimedOut);
            }
            result = &mut reader => result
                .map_err(|_| AppError::OcrFailed)??,
        };
        cancellation.check()?;
        if !status.success() || output.len() > MAX_OCR_TEXT_BYTES {
            return Err(AppError::OcrFailed);
        }
        let raw = String::from_utf8(output).map_err(|_| AppError::OcrFailed)?;
        let text = cleanup_text(&raw, mode);
        let score = score_text(&text);
        let warnings = if text.len() < 3 {
            vec![OcrWarning {
                code: "low_content".into(),
                message: "Very little text was recognized; review the result carefully.".into(),
            }]
        } else {
            Vec::new()
        };
        Ok(OcrCandidate {
            text,
            mean_confidence: None,
            preprocessing_variant: variant.id,
            warnings,
            score,
        })
    }
}

fn parse_languages(output: &[u8]) -> Result<Vec<String>, AppError> {
    let text = String::from_utf8_lossy(output);
    let languages: Vec<String> = text
        .lines()
        .map(str::trim)
        .filter(|line| {
            line.len() <= 32
                && line.chars().all(|character| {
                    character.is_ascii_alphanumeric() || matches!(character, '_' | '-')
                })
        })
        .map(str::to_owned)
        .collect();
    if !languages.iter().any(|language| language == "eng") {
        return Err(AppError::OcrLanguageMissing);
    }
    Ok(languages)
}

async fn terminate_child(child: &mut Child) -> Result<(), AppError> {
    match child.try_wait().map_err(|_| AppError::OcrFailed)? {
        Some(_) => Ok(()),
        None => {
            child.start_kill().map_err(|_| AppError::OcrFailed)?;
            child.wait().await.map_err(|_| AppError::OcrFailed)?;
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(unix)]
    use crate::models::PreprocessingVariant;
    #[cfg(unix)]
    use image::DynamicImage;
    #[cfg(unix)]
    use std::fs;
    #[cfg(unix)]
    use tempfile::tempdir;
    #[cfg(unix)]
    use tokio::time::{sleep, timeout};

    // Hosted Windows-only integration: unlike the direct executable smoke,
    // this exercises the shared bounded OCR probe and recognition pipeline.
    // The ignored test is invoked explicitly with a generated, public fixture.
    #[cfg(windows)]
    #[tokio::test]
    #[ignore = "requires staged checksum-verified OCR runtime and synthetic fixture"]
    async fn windows_staged_shared_engine_integration() {
        use crate::models::PreprocessingVariant;
        use std::path::PathBuf;

        let fixture = std::env::var_os("SCREENSHOT_OCR_WINDOWS_OCR_SMOKE_IMAGE")
            .map(PathBuf::from)
            .expect("synthetic Windows OCR fixture path required");
        // The ignored CI fixture may exercise the verified NSIS-extracted
        // resource tree; normal application runtime resolution never reads
        // this test-only environment variable.
        let root = std::env::var_os("SCREENSHOT_OCR_WINDOWS_OCR_RUNTIME_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("windows-ocr-runtime")
            });
        let executable = root.join("tesseract.exe");
        let tessdata = root.join("tessdata");
        assert!(executable.is_file() && tessdata.join("eng.traineddata").is_file());
        let engine = TesseractEngine::new(OcrRuntime {
            executable,
            tessdata_dir: Some(tessdata),
            working_directory: Some(root),
        });
        let cancellation = CancellationToken::new();
        let languages = engine.probe_english(&cancellation).await.unwrap();
        assert!(languages.iter().any(|language| language == "eng"));
        let variant = PreparedVariant {
            image: image::open(fixture).expect("load generated synthetic image"),
            id: PreprocessingVariant::Original,
        };
        let candidate = engine
            .recognize(&variant, TextMode::SingleLine, &cancellation)
            .await
            .unwrap();
        let normalized = candidate
            .text
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_ascii_uppercase();
        assert!(normalized.contains("SAMPLE 123"), "synthetic OCR mismatch");
    }

    // Hosted Windows CI proves cancellation of an already-running native child.
    #[cfg(windows)]
    #[tokio::test]
    #[ignore = "requires compiled synthetic Windows child fixture"]
    async fn windows_staged_shared_engine_cancellation() {
        use crate::models::PreprocessingVariant;
        use image::DynamicImage;
        use std::path::PathBuf;
        use std::time::Duration;
        use tokio::time::timeout;

        let executable = PathBuf::from(
            std::env::var_os("SCREENSHOT_OCR_WINDOWS_FAKE_TESSERACT")
                .expect("synthetic Windows executable required"),
        );
        let marker_dir = PathBuf::from(
            std::env::var_os("SCREENSHOT_OCR_WINDOWS_FAKE_MARKER_DIR")
                .expect("synthetic Windows marker directory required"),
        );
        // The synthetic child fails before its readiness marker unless the
        // shared engine pins the child CWD to this app-controlled directory.
        let engine = TesseractEngine::new(OcrRuntime {
            executable,
            tessdata_dir: None,
            working_directory: Some(marker_dir.clone()),
        });

        let cancellation = CancellationToken::new();
        let token = cancellation.clone();
        let probe_engine = engine.clone();
        let probe = tokio::spawn(async move { probe_engine.probe_english(&token).await });
        wait_for_windows_fake_start(&marker_dir.join("probe.started")).await;
        // Hold the exact child handle before cancellation so a reused PID
        // cannot make the process-exit assertion falsely pass.
        let probe_handle = windows_fake_process_handle(&marker_dir, "probe");
        cancellation.cancel();
        let probe_result = timeout(Duration::from_secs(4), probe)
            .await
            .expect("Windows language-probe cancellation timed out")
            .expect("Windows language-probe task panicked");
        assert!(matches!(probe_result, Err(AppError::CaptureCancelled)));
        assert_windows_fake_process_exited(&probe_handle);

        let cancellation = CancellationToken::new();
        let token = cancellation.clone();
        let variant = PreparedVariant {
            image: DynamicImage::new_rgb8(64, 32),
            id: PreprocessingVariant::Original,
        };
        let recognition = tokio::spawn(async move {
            engine
                .recognize(&variant, TextMode::SingleLine, &token)
                .await
        });
        wait_for_windows_fake_start(&marker_dir.join("recognize.started")).await;
        let recognition_handle = windows_fake_process_handle(&marker_dir, "recognize");
        cancellation.cancel();
        let recognition_result = timeout(Duration::from_secs(4), recognition)
            .await
            .expect("Windows OCR cancellation timed out")
            .expect("Windows OCR task panicked");
        assert!(matches!(
            recognition_result,
            Err(AppError::CaptureCancelled)
        ));
        assert_windows_fake_process_exited(&recognition_handle);
    }

    // Test-only native process inspection. The portable engine itself does
    // not depend on Windows APIs or expose any native process identifiers.
    #[cfg(windows)]
    fn windows_fake_process_handle(
        marker_dir: &std::path::Path,
        mode: &str,
    ) -> std::os::windows::io::OwnedHandle {
        use std::os::windows::io::{FromRawHandle, OwnedHandle};
        use windows_sys::Win32::System::Threading::OpenProcess;

        let pid = std::fs::read_to_string(marker_dir.join(format!("{mode}.pid")))
            .expect("synthetic Windows child PID marker")
            .trim()
            .parse::<u32>()
            .expect("numeric synthetic child PID");
        // SYNCHRONIZE alone permits waiting; no process mutation rights.
        let handle = unsafe { OpenProcess(0x0010_0000, 0, pid) };
        assert!(!handle.is_null(), "synthetic Windows child must still be running");
        // SAFETY: OpenProcess returned a distinct non-null owned handle.
        unsafe { OwnedHandle::from_raw_handle(handle as _) }
    }

    #[cfg(windows)]
    fn assert_windows_fake_process_exited(handle: &std::os::windows::io::OwnedHandle) {
        use std::os::windows::io::AsRawHandle;
        use windows_sys::Win32::Foundation::WAIT_OBJECT_0;
        use windows_sys::Win32::System::Threading::WaitForSingleObject;

        // A completed cancellation must leave the exact child terminated,
        // not merely mark the task cancelled while it keeps running.
        let result = unsafe { WaitForSingleObject(handle.as_raw_handle() as _, 0) };
        assert_eq!(
            result, WAIT_OBJECT_0,
            "synthetic Windows OCR child still running after cancellation"
        );
    }

    #[cfg(windows)]
    async fn wait_for_windows_fake_start(marker: &std::path::Path) {
        use std::time::Duration;
        use tokio::time::{sleep, Instant};

        let deadline = Instant::now() + Duration::from_secs(10);
        while !marker.is_file() {
            assert!(Instant::now() < deadline, "child did not start");
            sleep(Duration::from_millis(20)).await;
        }
    }

    #[cfg(unix)]
    fn write_executable(path: &std::path::Path, body: &str) {
        use std::os::unix::fs::PermissionsExt;

        fs::write(path, body).expect("write executable");
        fs::set_permissions(path, fs::Permissions::from_mode(0o700)).expect("make executable");
    }

    #[test]
    fn language_probe_filters_output_and_requires_english() {
        // Parsing is deterministic; the separate subprocess tests exercise
        // bounded execution, missing languages, and timeouts.
        let languages = parse_languages(b"List of available languages\neng\nosd\nbad/name\n")
            .expect("parse synthetic language list");
        assert!(languages.iter().any(|language| language == "eng"));
        assert!(languages.iter().any(|language| language == "osd"));
        assert!(!languages.iter().any(|language| language == "bad/name"));
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn language_probe_reports_missing_english() {
        let directory = tempdir().expect("tempdir");
        let executable = directory.path().join("fake-tesseract");
        write_executable(
            &executable,
            "#!/bin/sh\nif [ \"$1\" = \"--list-langs\" ]; then printf 'osd\n'; exit 0; fi\nexit 1\n",
        );
        let engine = TesseractEngine::new(OcrRuntime::new(executable, None));
        assert!(matches!(
            engine.probe_english(&CancellationToken::new()).await,
            Err(AppError::OcrLanguageMissing)
        ));
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn language_probe_rejects_oversized_stdout() {
        let directory = tempdir().expect("tempdir");
        let executable = directory.path().join("fake-tesseract");
        write_executable(
            &executable,
            "#!/bin/sh\nif [ \"$1\" = \"--list-langs\" ]; then head -c 70000 /dev/zero | tr '\\000' a; exit 0; fi\nexit 1\n",
        );
        let engine = TesseractEngine::new(OcrRuntime::new(executable, None));
        assert!(matches!(
            engine.probe_english(&CancellationToken::new()).await,
            Err(AppError::OcrEngineUnavailable)
        ));
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn language_probe_times_out_hanging_helper() {
        let directory = tempdir().expect("tempdir");
        let executable = directory.path().join("fake-tesseract");
        write_executable(
            &executable,
            "#!/bin/sh\nif [ \"$1\" = \"--list-langs\" ]; then exec sleep 30; fi\nexit 1\n",
        );
        let engine = TesseractEngine::new(OcrRuntime::new(executable, None));
        let result = timeout(
            LANGUAGE_PROBE_TIMEOUT + Duration::from_secs(2),
            engine.probe_english(&CancellationToken::new()),
        )
        .await
        .expect("language probe did not honor timeout");
        assert!(matches!(result, Err(AppError::OcrEngineUnavailable)));
    }

    #[tokio::test]
    async fn pre_cancelled_language_probe_exits_before_spawn_requirements() {
        let engine = TesseractEngine::new(OcrRuntime::new(
            std::path::PathBuf::from("/definitely/missing/tesseract"),
            None,
        ));
        let cancellation = CancellationToken::new();
        cancellation.cancel();
        assert!(matches!(
            engine.probe_english(&cancellation).await,
            Err(AppError::CaptureCancelled)
        ));
    }

    #[tokio::test]
    async fn pre_cancelled_recognition_never_requires_an_ocr_executable() {
        let engine = TesseractEngine::new(OcrRuntime::new(
            std::path::PathBuf::from("missing-ocr-fixture-executable"),
            None,
        ));
        let variant = PreparedVariant {
            image: image::DynamicImage::new_rgb8(2, 2),
            id: crate::models::PreprocessingVariant::Original,
        };
        let cancellation = CancellationToken::new();
        cancellation.cancel();
        assert!(matches!(
            engine
                .recognize(&variant, TextMode::SingleLine, &cancellation)
                .await,
            Err(AppError::CaptureCancelled)
        ));
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn cancellation_terminates_ocr_process() {
        let directory = tempdir().expect("tempdir");
        let executable = directory.path().join("slow-tesseract");
        // Synchronize with the synthetic process instead of relying on a
        // host-scheduler-dependent delay. Use exec so no shell descendant can
        // keep redirected pipe handles open after the child is terminated.
        write_executable(
            &executable,
            "#!/bin/sh\nprintf ready > ready.marker\nexec sleep 30\n",
        );
        let engine = TesseractEngine::new(OcrRuntime {
            executable,
            tessdata_dir: None,
            working_directory: Some(directory.path().to_path_buf()),
        });
        let variant = PreparedVariant {
            image: DynamicImage::new_luma8(1, 1),
            id: PreprocessingVariant::Original,
        };
        let cancellation = CancellationToken::new();
        let task_token = cancellation.clone();
        let task = tokio::spawn(async move {
            engine
                .recognize(&variant, TextMode::Terminal, &task_token)
                .await
        });
        // Hosted Linux runners may schedule this test alongside several
        // blocking process tests. A five-second startup bound is too tight
        // under contention; still fail promptly if OCR exited before ready.
        timeout(Duration::from_secs(15), async {
            while !directory.path().join("ready.marker").exists() {
                assert!(
                    !task.is_finished(),
                    "synthetic OCR task exited before the synthetic startup marker was written"
                );
                sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .expect("synthetic OCR process did not report startup");
        cancellation.cancel();
        let result = timeout(Duration::from_secs(2), task)
            .await
            .expect("OCR cancellation timed out")
            .expect("OCR task failed");
        assert!(
            matches!(result, Err(AppError::CaptureCancelled)),
            "unexpected cancellation result: {result:?}"
        );
    }
}
