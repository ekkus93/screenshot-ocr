use crate::cancellation::CancellationToken;
use crate::error::AppError;
use crate::models::CapturedImage;
use async_trait::async_trait;

#[async_trait]
pub trait CaptureBackend: Send + Sync {
    async fn capture_region(
        &self,
        cancellation: &CancellationToken,
    ) -> Result<CapturedImage, AppError>;
}
