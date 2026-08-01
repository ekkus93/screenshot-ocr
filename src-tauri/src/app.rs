use crate::capture::{CaptureBackend, EnvironmentInfo, EnvironmentProbe, GnomeScreenshotBackend};
use crate::error::AppError;
use crate::image_pipeline::prepare_variants;
use crate::models::{
    CaptureBackendPreference, CaptureJobId, CaptureRequest, OcrEngineId, OcrResult,
};
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
        Self {
            state: Arc::new(Mutex::new(CaptureStateMachine::default())),
            settings: Arc::new(SettingsStore::new(config_dir)),
        }
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

    async fn capture_inner(
        &self,
        job_id: CaptureJobId,
        request: CaptureRequest,
    ) -> Result<OcrResult, AppError> {
        let started = Instant::now();
        let settings = self.settings.load()?;
        let environment = EnvironmentProbe::probe()?;
        let backend = select_capture_backend(settings.capture_backend, &environment)?;
        let captured = backend.capture_region().await?;
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
            if candidates
                .last()
                .is_some_and(|candidate| candidate.score > 200)
            {
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

fn select_capture_backend(
    preference: CaptureBackendPreference,
    environment: &EnvironmentInfo,
) -> Result<Box<dyn CaptureBackend>, AppError> {
    match preference {
        CaptureBackendPreference::Auto | CaptureBackendPreference::Gnome => environment
            .gnome_screenshot
            .clone()
            .map(GnomeScreenshotBackend::new)
            .map(|backend| Box::new(backend) as Box<dyn CaptureBackend>)
            .ok_or(AppError::CaptureBackendUnavailable),
        CaptureBackendPreference::Portal => Err(AppError::CaptureBackendUnavailable),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn environment(gnome_screenshot: Option<PathBuf>) -> EnvironmentInfo {
        EnvironmentInfo {
            os_release: "Ubuntu".into(),
            desktop_environment: "GNOME".into(),
            session_type: "wayland".into(),
            gnome_screenshot,
            tesseract: None,
            portal_summary: "not proven".into(),
        }
    }

    #[test]
    fn explicit_portal_selection_fails_closed_until_supported() {
        assert!(matches!(
            select_capture_backend(CaptureBackendPreference::Portal, &environment(None)),
            Err(AppError::CaptureBackendUnavailable)
        ));
    }

    #[test]
    fn automatic_selection_requires_available_gnome_backend() {
        assert!(
            select_capture_backend(CaptureBackendPreference::Auto, &environment(None)).is_err()
        );
        assert!(select_capture_backend(
            CaptureBackendPreference::Auto,
            &environment(Some(PathBuf::from("/usr/bin/gnome-screenshot")))
        )
        .is_ok());
    }
}
