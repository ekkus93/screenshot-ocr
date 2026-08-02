use image::DynamicImage;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CaptureJobId(Uuid);

impl CaptureJobId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn is_valid(self) -> bool {
        !self.0.is_nil()
    }
}

impl Default for CaptureJobId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TextMode {
    Terminal,
    Document,
    SingleLine,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CopyPolicy {
    Preview,
    Immediate,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CaptureSource {
    MainWindow,
    Tray,
    Shortcut,
    CommandLine,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FrontendAppAction {
    StartCapture,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppActionEvent {
    pub action: FrontendAppAction,
    pub job_id: CaptureJobId,
    pub source: CaptureSource,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CaptureBackendPreference {
    Auto,
    Gnome,
    Portal,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureRequest {
    pub job_id: CaptureJobId,
    pub mode: TextMode,
    pub language: String,
    pub copy_policy: CopyPolicy,
    pub source: CaptureSource,
}

impl CaptureRequest {
    pub fn validate(&self) -> bool {
        self.job_id.is_valid() && self.language == "eng"
    }
}

#[derive(Debug)]
pub struct CapturedImage {
    pub image: DynamicImage,
    pub backend: CaptureBackendId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaptureBackendId {
    GnomeScreenshot,
    XdgPortal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OcrEngineId {
    Tesseract,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PreprocessingVariant {
    Original,
    GrayscaleContrast,
    InvertedGrayscale,
    Upscale2x,
    Upscale3x,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OcrWarning {
    pub code: String,
    pub message: String,
}

#[derive(Clone, Debug)]
pub struct OcrCandidate {
    pub text: String,
    pub mean_confidence: Option<f32>,
    pub preprocessing_variant: PreprocessingVariant,
    pub warnings: Vec<OcrWarning>,
    pub score: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OcrResult {
    pub job_id: CaptureJobId,
    pub text: String,
    pub mean_confidence: Option<f32>,
    pub backend: CaptureBackendId,
    pub engine: OcrEngineId,
    pub preprocessing_variant: PreprocessingVariant,
    pub warnings: Vec<OcrWarning>,
    pub copied: bool,
    pub elapsed_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capture_request_rejects_nil_job_ids() {
        let request = CaptureRequest {
            job_id: CaptureJobId(Uuid::nil()),
            mode: TextMode::Terminal,
            language: "eng".into(),
            copy_policy: CopyPolicy::Preview,
            source: CaptureSource::MainWindow,
        };
        assert!(!request.validate());
    }

    #[test]
    fn app_action_event_serializes_without_platform_details() {
        let event = AppActionEvent {
            action: FrontendAppAction::StartCapture,
            job_id: CaptureJobId::new(),
            source: CaptureSource::Shortcut,
        };
        let value = serde_json::to_value(event).expect("serialize");
        assert_eq!(value["action"], "startCapture");
        assert_eq!(value["source"], "shortcut");
        assert!(value.get("jobId").is_some());
    }
}
