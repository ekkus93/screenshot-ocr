use crate::capture::{
    CaptureBackend, EnvironmentInfo, EnvironmentProbe, GnomeScreenshotBackend,
    PortalScreenshotBackend,
};
use crate::error::AppError;
use crate::image_pipeline::prepare_variants;
use crate::models::{
    CaptureBackendId, CaptureBackendPreference, CaptureJobId, CaptureRequest, OcrEngineId,
    OcrResult,
};
use crate::ocr::{select_best_candidate, TesseractEngine};
use crate::settings::SettingsStore;
use crate::state::CaptureStateMachine;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct AppServices {
    pub state: Arc<Mutex<CaptureStateMachine>>,
    pub settings: Arc<SettingsStore>,
    startup_capture: Arc<AtomicBool>,
}

impl AppServices {
    pub fn new(config_dir: PathBuf, startup_capture: bool) -> Self {
        Self {
            state: Arc::new(Mutex::new(CaptureStateMachine::default())),
            settings: Arc::new(SettingsStore::new(config_dir)),
            startup_capture: Arc::new(AtomicBool::new(startup_capture)),
        }
    }

    pub fn take_startup_capture(&self) -> bool {
        self.startup_capture.swap(false, Ordering::AcqRel)
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
        let backend = select_capture_backend(settings.capture_backend, &environment).await?;
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

async fn select_capture_backend(
    preference: CaptureBackendPreference,
    environment: &EnvironmentInfo,
) -> Result<Box<dyn CaptureBackend>, AppError> {
    let portal_supported = match preference {
        CaptureBackendPreference::Gnome => false,
        CaptureBackendPreference::Auto => PortalScreenshotBackend::probe_area_support()
            .await
            .unwrap_or(false),
        CaptureBackendPreference::Portal => {
            PortalScreenshotBackend::probe_area_support().await?
        }
    };
    match choose_backend_id(
        preference,
        portal_supported,
        environment.gnome_screenshot.is_some(),
    )? {
        CaptureBackendId::XdgPortal => Ok(Box::new(PortalScreenshotBackend)),
        CaptureBackendId::GnomeScreenshot => environment
            .gnome_screenshot
            .clone()
            .map(GnomeScreenshotBackend::new)
            .map(|backend| Box::new(backend) as Box<dyn CaptureBackend>)
            .ok_or(AppError::CaptureBackendUnavailable),
    }
}

fn choose_backend_id(
    preference: CaptureBackendPreference,
    portal_supported: bool,
    gnome_available: bool,
) -> Result<CaptureBackendId, AppError> {
    match preference {
        CaptureBackendPreference::Portal if portal_supported => Ok(CaptureBackendId::XdgPortal),
        CaptureBackendPreference::Portal => Err(AppError::CaptureBackendUnavailable),
        CaptureBackendPreference::Gnome if gnome_available => {
            Ok(CaptureBackendId::GnomeScreenshot)
        }
        CaptureBackendPreference::Gnome => Err(AppError::CaptureBackendUnavailable),
        CaptureBackendPreference::Auto if portal_supported => Ok(CaptureBackendId::XdgPortal),
        CaptureBackendPreference::Auto if gnome_available => {
            Ok(CaptureBackendId::GnomeScreenshot)
        }
        CaptureBackendPreference::Auto => Err(AppError::CaptureBackendUnavailable),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn startup_capture_is_consumed_once() {
        let directory = tempdir().expect("tempdir");
        let services = AppServices::new(directory.path().to_path_buf(), true);
        assert!(services.take_startup_capture());
        assert!(!services.take_startup_capture());
    }

    #[test]
    fn automatic_selection_prefers_proven_portal_area_capture() {
        assert_eq!(
            choose_backend_id(CaptureBackendPreference::Auto, true, true)
                .expect("portal backend"),
            CaptureBackendId::XdgPortal
        );
    }

    #[test]
    fn automatic_selection_falls_back_before_opening_a_selector() {
        assert_eq!(
            choose_backend_id(CaptureBackendPreference::Auto, false, true)
                .expect("GNOME backend"),
            CaptureBackendId::GnomeScreenshot
        );
    }

    #[test]
    fn explicit_portal_selection_fails_closed_without_area_support() {
        assert!(matches!(
            choose_backend_id(CaptureBackendPreference::Portal, false, true),
            Err(AppError::CaptureBackendUnavailable)
        ));
    }

    #[test]
    fn automatic_selection_rejects_missing_backends() {
        assert!(matches!(
            choose_backend_id(CaptureBackendPreference::Auto, false, false),
            Err(AppError::CaptureBackendUnavailable)
        ));
    }
}
