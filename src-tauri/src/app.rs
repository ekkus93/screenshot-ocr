use crate::cancellation::CancellationToken;
use crate::capture::{
    CaptureBackend, EnvironmentInfo, EnvironmentProbe, GnomeScreenshotBackend,
    PortalScreenshotBackend,
};
use crate::diagnostics::RuntimeDiagnostics;
use crate::error::AppError;
use crate::image_pipeline::prepare_variants;
use crate::models::{
    AppActionEvent, CaptureBackendId, CaptureBackendPreference, CaptureJobId, CaptureRequest,
    OcrEngineId, OcrResult,
};
use crate::ocr::{select_best_candidate, TesseractEngine};
use crate::settings::SettingsStore;
use crate::state::CaptureStateMachine;
use std::path::PathBuf;
use std::sync::{Arc, Mutex as StdMutex};
use std::time::Instant;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct AppServices {
    pub state: Arc<Mutex<CaptureStateMachine>>,
    pub settings: Arc<SettingsStore>,
    pub runtime_diagnostics: Arc<RuntimeDiagnostics>,
    pending_app_action: Arc<StdMutex<Option<AppActionEvent>>>,
}

impl AppServices {
    pub fn new(config_dir: PathBuf) -> Self {
        Self {
            state: Arc::new(Mutex::new(CaptureStateMachine::default())),
            settings: Arc::new(SettingsStore::new(config_dir)),
            runtime_diagnostics: Arc::new(RuntimeDiagnostics::default()),
            pending_app_action: Arc::new(StdMutex::new(None)),
        }
    }

    pub fn queue_app_action(&self, action: AppActionEvent) -> Result<(), AppError> {
        let mut pending = self
            .pending_app_action
            .lock()
            .map_err(|_| AppError::Internal)?;
        if pending.is_some() {
            return Err(AppError::CaptureAlreadyActive);
        }
        *pending = Some(action);
        Ok(())
    }

    pub fn take_app_action(&self) -> Option<AppActionEvent> {
        self.pending_app_action
            .lock()
            .ok()
            .and_then(|mut pending| pending.take())
    }

    pub fn clear_app_action(&self, job_id: CaptureJobId) {
        if let Ok(mut pending) = self.pending_app_action.lock() {
            if pending.as_ref().is_some_and(|action| action.job_id == job_id) {
                *pending = None;
            }
        }
    }

    pub async fn capture(&self, request: CaptureRequest) -> Result<OcrResult, AppError> {
        if !request.validate() {
            return Err(AppError::SettingsInvalid);
        }
        let job_id = request.job_id;
        let cancellation = self.state.lock().await.begin(job_id)?;
        self.capture_inner(job_id, request, &cancellation).await
    }

    async fn capture_inner(
        &self,
        job_id: CaptureJobId,
        request: CaptureRequest,
        cancellation: &CancellationToken,
    ) -> Result<OcrResult, AppError> {
        let started = Instant::now();
        cancellation.check()?;
        let settings = self.settings.load()?;
        let environment = EnvironmentProbe::probe()?;
        let backend =
            select_capture_backend(settings.capture_backend, &environment, cancellation).await?;
        let captured = backend.capture_region(cancellation).await?;
        cancellation.check()?;
        let engine = TesseractEngine::from_environment(&environment)?;
        engine.probe_english()?;
        cancellation.check()?;
        let variants = prepare_variants(&captured.image);
        let mut candidates = Vec::new();
        for variant in variants.iter().take(4) {
            cancellation.check()?;
            match engine.recognize(variant, request.mode, cancellation).await {
                Ok(candidate) => candidates.push(candidate),
                Err(AppError::CaptureCancelled) => return Err(AppError::CaptureCancelled),
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
        cancellation.check()?;
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
    cancellation: &CancellationToken,
) -> Result<Box<dyn CaptureBackend>, AppError> {
    cancellation.check()?;
    let portal_supported = match preference {
        CaptureBackendPreference::Gnome => false,
        CaptureBackendPreference::Auto => {
            match PortalScreenshotBackend::probe_area_support(cancellation).await {
                Ok(supported) => supported,
                Err(AppError::CaptureCancelled) => return Err(AppError::CaptureCancelled),
                Err(_) => false,
            }
        }
        CaptureBackendPreference::Portal => {
            PortalScreenshotBackend::probe_area_support(cancellation).await?
        }
    };
    cancellation.check()?;
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
        CaptureBackendPreference::Gnome if gnome_available => Ok(CaptureBackendId::GnomeScreenshot),
        CaptureBackendPreference::Gnome => Err(AppError::CaptureBackendUnavailable),
        CaptureBackendPreference::Auto if portal_supported => Ok(CaptureBackendId::XdgPortal),
        CaptureBackendPreference::Auto if gnome_available => Ok(CaptureBackendId::GnomeScreenshot),
        CaptureBackendPreference::Auto => Err(AppError::CaptureBackendUnavailable),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CaptureSource, FrontendAppAction};
    use tempfile::tempdir;

    #[test]
    fn pending_app_action_is_consumed_once() {
        let directory = tempdir().expect("tempdir");
        let services = AppServices::new(directory.path().to_path_buf());
        let job_id = CaptureJobId::new();
        services
            .queue_app_action(AppActionEvent {
                action: FrontendAppAction::StartCapture,
                job_id,
                source: CaptureSource::CommandLine,
            })
            .expect("queue action");
        assert_eq!(
            services.take_app_action().map(|action| action.job_id),
            Some(job_id)
        );
        assert!(services.take_app_action().is_none());
    }

    #[test]
    fn pending_app_action_does_not_overwrite_an_existing_request() {
        let directory = tempdir().expect("tempdir");
        let services = AppServices::new(directory.path().to_path_buf());
        services
            .queue_app_action(AppActionEvent {
                action: FrontendAppAction::StartCapture,
                job_id: CaptureJobId::new(),
                source: CaptureSource::Tray,
            })
            .expect("first action");
        assert!(matches!(
            services.queue_app_action(AppActionEvent {
                action: FrontendAppAction::StartCapture,
                job_id: CaptureJobId::new(),
                source: CaptureSource::Shortcut,
            }),
            Err(AppError::CaptureAlreadyActive)
        ));
    }

    #[test]
    fn automatic_selection_prefers_proven_portal_area_capture() {
        assert_eq!(
            choose_backend_id(CaptureBackendPreference::Auto, true, true).expect("portal backend"),
            CaptureBackendId::XdgPortal
        );
    }

    #[test]
    fn automatic_selection_falls_back_before_opening_a_selector() {
        assert_eq!(
            choose_backend_id(CaptureBackendPreference::Auto, false, true).expect("GNOME backend"),
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
