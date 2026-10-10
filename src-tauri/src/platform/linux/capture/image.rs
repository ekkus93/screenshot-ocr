use crate::error::AppError;
use crate::models::{CaptureBackendId, CapturedImage};
use image::ImageReader;
use std::io::Cursor;

const MAX_ENCODED_BYTES: usize = 20 * 1024 * 1024;
const MAX_WIDTH: u32 = 12_000;
const MAX_HEIGHT: u32 = 12_000;
const MAX_PIXELS: u64 = 60_000_000;

pub(super) fn decode_captured_image(
    bytes: &[u8],
    backend: CaptureBackendId,
) -> Result<CapturedImage, AppError> {
    if bytes.is_empty() {
        return Err(AppError::CaptureImageInvalid);
    }
    if bytes.len() > MAX_ENCODED_BYTES {
        return Err(AppError::CaptureTooLarge);
    }
    let reader = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|_| AppError::CaptureImageInvalid)?;
    let (width, height) = reader
        .into_dimensions()
        .map_err(|_| AppError::CaptureImageInvalid)?;
    enforce_dimensions(width, height)?;
    let image = image::load_from_memory(bytes).map_err(|_| AppError::CaptureImageInvalid)?;
    Ok(CapturedImage {
        image,
        backend,
        cleanup_failed: false,
    })
}

fn enforce_dimensions(width: u32, height: u32) -> Result<(), AppError> {
    if width == 0 || height == 0 {
        return Err(AppError::CaptureImageInvalid);
    }
    if width > MAX_WIDTH || height > MAX_HEIGHT || u64::from(width) * u64::from(height) > MAX_PIXELS
    {
        return Err(AppError::CaptureTooLarge);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_and_oversized_inputs() {
        assert!(matches!(
            decode_captured_image(&[], CaptureBackendId::GnomeScreenshot),
            Err(AppError::CaptureImageInvalid)
        ));
        assert!(matches!(
            enforce_dimensions(20_000, 20_000),
            Err(AppError::CaptureTooLarge)
        ));
    }
}
