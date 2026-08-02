use crate::error::AppError;
use crate::models::{CaptureBackendId, CapturedImage, PreprocessingVariant};
use image::imageops::FilterType;
use image::{DynamicImage, ImageFormat, ImageReader};
use std::io::Cursor;

const MAX_ENCODED_BYTES: usize = 20 * 1024 * 1024;
const MAX_WIDTH: u32 = 12_000;
const MAX_HEIGHT: u32 = 12_000;
const MAX_PIXELS: u64 = 60_000_000;

#[derive(Debug)]
pub struct PreparedVariant {
    pub image: DynamicImage,
    pub id: PreprocessingVariant,
}

pub fn decode_captured_image(
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
    Ok(CapturedImage { image, backend })
}

pub fn prepare_variants(image: &DynamicImage) -> Vec<PreparedVariant> {
    let mut variants = Vec::with_capacity(4);
    variants.push(PreparedVariant {
        image: image.clone(),
        id: PreprocessingVariant::Original,
    });
    let grayscale = image.grayscale();
    variants.push(PreparedVariant {
        image: grayscale.clone(),
        id: PreprocessingVariant::Grayscale,
    });
    if mean_luminance(&grayscale) < 110.0 {
        let mut inverted = grayscale.clone();
        inverted.invert();
        variants.push(PreparedVariant {
            image: inverted,
            id: PreprocessingVariant::InvertedGrayscale,
        });
    }
    if let Some((width, height)) = checked_scaled_dimensions(image.width(), image.height(), 2) {
        if enforce_dimensions(width, height).is_ok() {
            variants.push(PreparedVariant {
                image: grayscale.resize(width, height, FilterType::CatmullRom),
                id: PreprocessingVariant::Upscale2x,
            });
        }
    }
    variants
}

pub fn encode_png(image: &DynamicImage) -> Result<Vec<u8>, AppError> {
    let mut output = Cursor::new(Vec::new());
    image
        .write_to(&mut output, ImageFormat::Png)
        .map_err(|_| AppError::OcrFailed)?;
    Ok(output.into_inner())
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

fn checked_scaled_dimensions(width: u32, height: u32, factor: u32) -> Option<(u32, u32)> {
    Some((width.checked_mul(factor)?, height.checked_mul(factor)?))
}

fn mean_luminance(image: &DynamicImage) -> f64 {
    let gray = image.to_luma8();
    let total: u64 = gray.pixels().map(|pixel| u64::from(pixel[0])).sum();
    if gray.is_empty() {
        0.0
    } else {
        total as f64 / gray.len() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgb};

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

    #[test]
    fn dark_images_receive_inverted_variant() {
        let image = DynamicImage::ImageRgb8(ImageBuffer::from_pixel(10, 10, Rgb([5, 5, 5])));
        assert!(prepare_variants(&image)
            .iter()
            .any(|variant| variant.id == PreprocessingVariant::InvertedGrayscale));
    }

    #[test]
    fn grayscale_variant_is_named_truthfully() {
        let image = DynamicImage::ImageRgb8(ImageBuffer::from_pixel(10, 10, Rgb([200, 200, 200])));
        assert!(prepare_variants(&image)
            .iter()
            .any(|variant| variant.id == PreprocessingVariant::Grayscale));
    }

    #[test]
    fn generated_upscale_dimensions_are_rechecked() {
        let (width, height) =
            checked_scaled_dimensions(6_000, 6_000, 2).expect("scaled dimensions");
        assert!(matches!(
            enforce_dimensions(width, height),
            Err(AppError::CaptureTooLarge)
        ));
    }
}
