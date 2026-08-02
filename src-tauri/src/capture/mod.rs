mod environment;
mod gnome;
mod portal;

use crate::error::AppError;
use crate::models::CapturedImage;
use async_trait::async_trait;

pub use environment::{EnvironmentInfo, EnvironmentProbe};
pub use gnome::GnomeScreenshotBackend;
pub use portal::PortalScreenshotBackend;

#[async_trait]
pub trait CaptureBackend: Send + Sync {
    async fn capture_region(&self) -> Result<CapturedImage, AppError>;
}
