use crate::cancellation::CancellationToken;
use crate::capture::EnvironmentInfo;
use crate::error::AppError;
use crate::image_pipeline::{encode_png, PreparedVariant};
use crate::models::{OcrCandidate, OcrWarning, TextMode};
use crate::ocr::cleanup::{cleanup_text, score_text};
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::{Child, Command};
use tokio::time::{sleep_until, Instant};

const OCR_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_OCR_TEXT_BYTES: usize = 1_000_000;

#[derive(Clone, Debug)]
pub struct TesseractEngine {
    executable: std::path::PathBuf,
}

impl TesseractEngine {
    pub fn from_environment(environment: &EnvironmentInfo) -> Result<Self, AppError> {
        environment
            .tesseract
            .clone()
            .map(|executable| Self { executable })
            .ok_or(AppError::OcrEngineUnavailable)
    }

    pub fn probe_english(&self) -> Result<Vec<String>, AppError> {
        let output = std::process::Command::new(&self.executable)
            .arg("--list-langs")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .map_err(|_| AppError::OcrEngineUnavailable)?;
        if !output.status.success() || output.stdout.len() > 64 * 1024 {
            return Err(AppError::OcrEngineUnavailable);
        }
        let text = String::from_utf8_lossy(&output.stdout);
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

    pub async fn recognize(
        &self,
        variant: &PreparedVariant,
        mode: TextMode,
        cancellation: &CancellationToken,
    ) -> Result<OcrCandidate, AppError> {
        cancellation.check()?;
        let deadline = Instant::now() + OCR_TIMEOUT;
        let png = encode_png(&variant.image)?;
        let page_segmentation = match mode {
            TextMode::Terminal => "6",
            TextMode::Document => "3",
            TextMode::SingleLine => "7",
        };
        let mut child = Command::new(&self.executable)
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
    use crate::models::PreprocessingVariant;
    use image::DynamicImage;
    use std::fs;
    use tempfile::tempdir;
    use tokio::time::{sleep, timeout};

    #[cfg(unix)]
    #[tokio::test]
    async fn cancellation_terminates_ocr_process() {
        use std::os::unix::fs::PermissionsExt;

        let directory = tempdir().expect("tempdir");
        let executable = directory.path().join("slow-tesseract");
        fs::write(&executable, "#!/bin/sh\nsleep 30\n").expect("write executable");
        fs::set_permissions(&executable, fs::Permissions::from_mode(0o700))
            .expect("make executable");
        let engine = TesseractEngine { executable };
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
        sleep(Duration::from_millis(50)).await;
        cancellation.cancel();
        let result = timeout(Duration::from_secs(2), task)
            .await
            .expect("OCR cancellation timed out")
            .expect("OCR task failed");
        assert!(matches!(result, Err(AppError::CaptureCancelled)));
    }
}
