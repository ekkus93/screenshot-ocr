use crate::capture::EnvironmentInfo;
use crate::error::AppError;
use crate::image_pipeline::{encode_png, PreparedVariant};
use crate::models::{OcrCandidate, OcrWarning, TextMode};
use crate::ocr::cleanup::{cleanup_text, score_text};
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;
use tokio::time::timeout;

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
    ) -> Result<OcrCandidate, AppError> {
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
        let mut stdin = child.stdin.take().ok_or(AppError::OcrFailed)?;
        stdin
            .write_all(&png)
            .await
            .map_err(|_| AppError::OcrFailed)?;
        drop(stdin);

        let stdout = child.stdout.take().ok_or(AppError::OcrFailed)?;
        let read_stdout = async move {
            let mut limited = stdout.take((MAX_OCR_TEXT_BYTES + 1) as u64);
            let mut bytes = Vec::new();
            limited
                .read_to_end(&mut bytes)
                .await
                .map_err(|_| AppError::OcrFailed)?;
            Ok::<Vec<u8>, AppError>(bytes)
        };
        let wait_for_output = async {
            let (status, bytes) = tokio::try_join!(
                async { child.wait().await.map_err(|_| AppError::OcrFailed) },
                read_stdout
            )?;
            Ok::<_, AppError>((status, bytes))
        };
        let (status, output) = timeout(OCR_TIMEOUT, wait_for_output)
            .await
            .map_err(|_| AppError::OcrTimedOut)??;
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
