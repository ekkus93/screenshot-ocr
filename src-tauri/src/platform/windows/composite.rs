//! Pure composition of monitor-local pixel tiles into a virtual-desktop image.
use super::geometry::{MonitorTile, Rect};
use crate::error::AppError;
use image::{GenericImage, RgbaImage};

/// Preserve the physical position of each monitor tile. Uncovered areas
/// remain transparent, and malformed tile dimensions fail closed.
pub fn compose(
    selection: Rect,
    tiles: impl IntoIterator<Item = (MonitorTile, RgbaImage)>,
) -> Result<RgbaImage, AppError> {
    selection.rgba_len()?;
    let width = u32::try_from(selection.width()).map_err(|_| AppError::CaptureTooLarge)?;
    let height = u32::try_from(selection.height()).map_err(|_| AppError::CaptureTooLarge)?;
    let mut result = RgbaImage::new(width, height);
    for (tile, image) in tiles {
        let tile_width =
            u32::try_from(tile.source.width()).map_err(|_| AppError::CaptureImageInvalid)?;
        let tile_height =
            u32::try_from(tile.source.height()).map_err(|_| AppError::CaptureImageInvalid)?;
        if image.width() != tile_width || image.height() != tile_height {
            return Err(AppError::CaptureImageInvalid);
        }
        result
            .copy_from(&image, tile.offset_x, tile.offset_y)
            .map_err(|_| AppError::CaptureImageInvalid)?;
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::windows::geometry::monitor_tiles;
    use image::Rgba;

    fn rect(left: i32, top: i32, right: i32, bottom: i32) -> Rect {
        Rect {
            left,
            top,
            right,
            bottom,
        }
    }

    #[test]
    fn negative_origin_and_monitor_gap_are_preserved() {
        let selection = rect(-2, 0, 3, 2);
        let monitors = [rect(-2, 0, 0, 2), rect(1, 0, 3, 2)];
        let tiles = monitor_tiles(selection, &monitors);
        let pixels = tiles.into_iter().enumerate().map(|(index, tile)| {
            let color = if index == 0 {
                Rgba([255, 0, 0, 255])
            } else {
                Rgba([0, 255, 0, 255])
            };
            (
                tile,
                RgbaImage::from_pixel(
                    tile.source.width() as u32,
                    tile.source.height() as u32,
                    color,
                ),
            )
        });
        let image = compose(selection, pixels).unwrap();
        assert_eq!(image.dimensions(), (5, 2));
        assert_eq!(image.get_pixel(0, 0).0, [255, 0, 0, 255]);
        assert_eq!(image.get_pixel(2, 0).0, [0, 0, 0, 0]);
        assert_eq!(image.get_pixel(4, 0).0, [0, 255, 0, 255]);
    }

    #[test]
    fn rejects_mismatched_tile_dimensions() {
        let selection = rect(0, 0, 10, 10);
        let tile = monitor_tiles(selection, &[selection])[0];
        assert!(matches!(
            compose(selection, [(tile, RgbaImage::new(1, 1))]),
            Err(AppError::CaptureImageInvalid)
        ));
    }
}
