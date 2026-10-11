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
    let mut covered = Vec::<Rect>::new();
    for (tile, image) in tiles {
        // Overlapping monitor tiles would overwrite previously captured pixels
        // and make multi-monitor OCR results dependent on enumeration order.
        if covered.iter().any(|previous| previous.intersect(tile.source).is_some()) {
            return Err(AppError::CaptureImageInvalid);
        }
        let tile_width =
            u32::try_from(tile.source.width()).map_err(|_| AppError::CaptureImageInvalid)?;
        let tile_height =
            u32::try_from(tile.source.height()).map_err(|_| AppError::CaptureImageInvalid)?;
        if image.width() != tile_width || image.height() != tile_height {
            return Err(AppError::CaptureImageInvalid);
        }
        // A tile must describe its actual location in the selected physical
        // desktop region; an in-bounds but incorrect offset silently corrupts
        // multi-monitor OCR input.
        if tile.source.intersect(selection) != Some(tile.source) {
            return Err(AppError::CaptureImageInvalid);
        }
        let expected_x = u32::try_from(i64::from(tile.source.left) - i64::from(selection.left))
            .map_err(|_| AppError::CaptureImageInvalid)?;
        let expected_y = u32::try_from(i64::from(tile.source.top) - i64::from(selection.top))
            .map_err(|_| AppError::CaptureImageInvalid)?;
        if (tile.offset_x, tile.offset_y) != (expected_x, expected_y) {
            return Err(AppError::CaptureImageInvalid);
        }
        result
            .copy_from(&image, tile.offset_x, tile.offset_y)
            .map_err(|_| AppError::CaptureImageInvalid)?;
        covered.push(tile.source);
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
    fn rejects_in_bounds_but_incorrect_monitor_offset() {
        let selection = rect(-10, -10, 10, 10);
        let mut tile = monitor_tiles(selection, &[rect(-10, -10, -8, -8)])[0];
        tile.offset_x = 1;
        assert!(matches!(
            compose(selection, [(tile, RgbaImage::new(2, 2))]),
            Err(AppError::CaptureImageInvalid)
        ));
    }

    #[test]
    fn rejects_monitor_source_outside_selection_even_if_offset_fits() {
        let selection = rect(0, 0, 10, 10);
        let tile = MonitorTile {
            source: rect(-1, 0, 1, 2),
            offset_x: 0,
            offset_y: 0,
        };
        assert!(matches!(
            compose(selection, [(tile, RgbaImage::new(2, 2))]),
            Err(AppError::CaptureImageInvalid)
        ));
    }

    #[test]
    fn rejects_overlapping_monitor_tiles_even_with_valid_offsets() {
        let selection = rect(-5, 0, 5, 4);
        let tiles = monitor_tiles(selection, &[rect(-5, 0, 1, 4), rect(0, 0, 5, 4)]);
        let images = tiles.into_iter().map(|tile| {
            (tile, RgbaImage::new(tile.source.width() as u32, tile.source.height() as u32))
        });
        assert!(matches!(
            compose(selection, images),
            Err(AppError::CaptureImageInvalid)
        ));
    }

    #[test]
    fn rejects_duplicate_monitor_tiles() {
        let selection = rect(0, 0, 4, 4);
        let tile = monitor_tiles(selection, &[selection])[0];
        assert!(matches!(
            compose(selection, [(tile, RgbaImage::new(4, 4)), (tile, RgbaImage::new(4, 4))]),
            Err(AppError::CaptureImageInvalid)
        ));
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
