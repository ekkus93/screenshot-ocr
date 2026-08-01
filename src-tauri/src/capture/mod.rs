mod environment;
mod gnome;

use crate::error::AppError;
use crate::models::CapturedImage;
use async_trait::async_trait;

pub use environment::{EnvironmentInfo, EnvironmentProbe};
pub use gnome::GnomeScreenshotBackend;

#[async_trait]
pub trait CaptureBackend: Send + Sync {
    async fn capture_region(&self) -> Result<CapturedImage, AppError>;
}
