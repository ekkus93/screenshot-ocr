use crate::capture::{EnvironmentProbe, GnomeScreenshotBackend};
use crate::error::AppError;
use crate::image_pipeline::prepare_variants;
use crate::models::{CaptureRequest, OcrEngineId, OcrResult};
use crate::ocr::{select_best_candidate, TesseractEngine};
use crate::settings::SettingsStore;
use crate::state::CaptureStateMachine;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct AppServices {
    pub state: Arc<Mutex<CaptureStateMachine>>,
    pub settings: Arc<SettingsStore>,
}

impl AppServices {
    pub fn new(config_dir: PathBuf) -> Self {
        Self { state: Arc::new(Mutex::new(CaptureStateMachine::default())), settings: Arc::new(SettingsStore::new(config_dir)) }
    }

    pub async fn capture(&self, request: CaptureRequest) -> Result<OcrResult, AppError> {
        if !request.validate() {
            return Err(AppError::SettingsInvalid);
        }
        let job_id = self.state.lock().await.begin()?;
        let result = self.capture_inner(job_id, request).await;
        self.state.lock().await.finish(job_id);
        result
    }

    async fn capture_inner(&self, job_id: crate::models::CaptureJobId, request: CaptureRequest) -> Result<OcrResult, AppError> {
        let started = Instant::now();
        let environment = EnvironmentProbe::probe()?;
        let executable = environment.gnome_screenshot.clone().ok_or(AppError::CaptureBackendUnavailable)?;
        let captured = GnomeScreenshotBackend::new(executable).capture_region().await?;
        let engine = TesseractEngine::from_environment(&environment)?;
        engine.probe_english()?;
        let variants = prepare_variants(&captured.image);
        let mut candidates = Vec::new();
        for variant in variants.iter().take(4) {
            match engine.recognize(variant, request.mode).await {
                Ok(candidate) => candidates.push(candidate),
                Err(AppError::OcrTimedOut) => return Err(AppError::OcrTimedOut),
                Err(_) => continue,
            }
            if candidates.last().is_some_and(|candidate| candidate.score > 200) {
                break;
            }
        }
        let candidate = select_best_candidate(candidates)?;
        Ok(OcrResult {
            job_id,
            text: candidate.text,
            mean_confidence: candidate.mean_confidence,
            backend: captured.backend,
            engine: OcrEngineId::Tesseract,
            preprocessing_variant: candidate.preprocessing_variant,
            warnings: candidate.warnings,
            copied: false,
            elapsed_ms: started.elapsed().as_millis().try_into().unwrap_or(u64::MAX),
        })
    }
}
