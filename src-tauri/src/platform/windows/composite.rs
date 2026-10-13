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
        if covered
            .iter()
            .any(|previous| previous.intersect(tile.source).is_some())
        {
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
    // A selection outside every active display must never produce an empty,
    // fully transparent OCR image even if this pure composer is called directly.
    if covered.is_empty() {
        return Err(AppError::CaptureImageInvalid);
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
    fn rejects_selection_without_any_display_tiles() {
        let selection = rect(0, 0, 20, 20);
        assert!(matches!(
            compose(selection, std::iter::empty()),
            Err(AppError::CaptureImageInvalid)
        ));
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
    fn mixed_dpi_cross_monitor_composition_uses_physical_offsets_once() {
        use crate::platform::windows::geometry::logical_to_physical;

        // Negative-origin monitor at 125%; primary monitor at 200%.
        // Tile offsets must not be scaled after logical input conversion.
        let left = logical_to_physical(4, 125).unwrap();
        let right = logical_to_physical(3, 200).unwrap();
        assert_eq!((left, right), (5, 6));
        let selection = rect(-left, 0, right, 2);
        let monitors = [rect(-1920, 0, 0, 1080), rect(0, 0, 2560, 1440)];
        let tiles = monitor_tiles(selection, &monitors);
        assert_eq!(tiles.len(), 2);
        let images = tiles.into_iter().enumerate().map(|(index, tile)| {
            let color = if index == 0 {
                Rgba([10, 20, 30, 255])
            } else {
                Rgba([40, 50, 60, 255])
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
        let image = compose(selection, images).unwrap();
        assert_eq!(image.dimensions(), (11, 2));
        assert_eq!(image.get_pixel(4, 1).0, [10, 20, 30, 255]);
        assert_eq!(image.get_pixel(5, 1).0, [40, 50, 60, 255]);
        assert_eq!(image.get_pixel(10, 1).0, [40, 50, 60, 255]);
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
            (
                tile,
                RgbaImage::new(tile.source.width() as u32, tile.source.height() as u32),
            )
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
            compose(
                selection,
                [(tile, RgbaImage::new(4, 4)), (tile, RgbaImage::new(4, 4))]
            ),
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
