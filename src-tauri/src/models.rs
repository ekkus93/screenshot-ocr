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
pub enum CaptureBackendPreference {
    Auto,
    Gnome,
    Portal,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureRequest {
    pub mode: TextMode,
    pub language: String,
    pub copy_policy: CopyPolicy,
    pub source: CaptureSource,
}

impl CaptureRequest {
    pub fn validate(&self) -> bool {
        self.language == "eng"
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
