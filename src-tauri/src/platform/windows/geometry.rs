//! Physical-pixel virtual-desktop geometry for the Windows selector.
//!
//! Windows mouse events are already physical pixels under Per-Monitor-v2 DPI
//! awareness. The conversion helper is for explicit logical-coordinate inputs,
//! not for applying scaling to raw selector events.
use crate::error::AppError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl Rect {
    pub fn from_drag(start: Point, end: Point) -> Result<Self, AppError> {
        let rect = Self {
            left: start.x.min(end.x),
            top: start.y.min(end.y),
            right: start.x.max(end.x),
            bottom: start.y.max(end.y),
        };
        // A single-pixel or empty drag is not a useful OCR selection.
        if rect.width() < 2 || rect.height() < 2 {
            return Err(AppError::CaptureImageInvalid);
        }
        rect.rgba_len()?;
        Ok(rect)
    }

    pub fn width(self) -> i64 {
        i64::from(self.right) - i64::from(self.left)
    }

    pub fn height(self) -> i64 {
        i64::from(self.bottom) - i64::from(self.top)
    }

    // Used by the monitor-stitching stage; pure tests already cover edge intersections.
    #[allow(dead_code)]
    pub fn intersect(self, other: Self) -> Option<Self> {
        let intersection = Self {
            left: self.left.max(other.left),
            top: self.top.max(other.top),
            right: self.right.min(other.right),
            bottom: self.bottom.min(other.bottom),
        };
        (intersection.width() > 0 && intersection.height() > 0).then_some(intersection)
    }

    pub fn rgba_len(self) -> Result<usize, AppError> {
        let width = self.width();
        let height = self.height();
        if width <= 0 || height <= 0 {
            return Err(AppError::CaptureImageInvalid);
        }
        if width > 12_000 || height > 12_000 || width * height > 60_000_000 {
            return Err(AppError::CaptureTooLarge);
        }
        let bytes = width
            .checked_mul(height)
            .and_then(|pixels| pixels.checked_mul(4))
            .ok_or(AppError::CaptureTooLarge)?;
        usize::try_from(bytes).map_err(|_| AppError::CaptureTooLarge)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MonitorTile {
    pub source: Rect,
    pub offset_x: u32,
    pub offset_y: u32,
}

/// Intersect the selection with each monitor in physical virtual-screen space.
/// Uncovered virtual-screen gaps remain transparent in the composite buffer.
// Planned for native monitor enumeration/compositing; keep behavior unit-tested.
#[allow(dead_code)]
pub fn monitor_tiles(selection: Rect, monitors: &[Rect]) -> Vec<MonitorTile> {
    monitors
        .iter()
        .filter_map(|monitor| {
            let source = selection.intersect(*monitor)?;
            let offset_x =
                u32::try_from(i64::from(source.left) - i64::from(selection.left)).ok()?;
            let offset_y = u32::try_from(i64::from(source.top) - i64::from(selection.top)).ok()?;
            Some(MonitorTile {
                source,
                offset_x,
                offset_y,
            })
        })
        .collect()
}

/// Round logical pixels to physical pixels, half away from zero.
/// Selector pointer events must not be passed through this conversion.
// The selector currently takes physical pointer events; this helper is for
// explicit logical inputs in a later integration and is covered by unit tests.
#[allow(dead_code)]
pub fn logical_to_physical(value: i32, scale_percent: u32) -> Result<i32, AppError> {
    if !(100..=400).contains(&scale_percent) {
        return Err(AppError::CaptureImageInvalid);
    }
    let scaled = i64::from(value) * i64::from(scale_percent);
    let rounded = if scaled < 0 {
        (scaled - 50) / 100
    } else {
        (scaled + 50) / 100
    };
    i32::try_from(rounded).map_err(|_| AppError::CaptureTooLarge)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn point(x: i32, y: i32) -> Point {
        Point { x, y }
    }

    fn rect(left: i32, top: i32, right: i32, bottom: i32) -> Rect {
        Rect {
            left,
            top,
            right,
            bottom,
        }
    }

    #[test]
    fn drag_normalizes_negative_coordinates_and_reverse_direction() {
        assert_eq!(
            Rect::from_drag(point(100, 200), point(-400, -200)).unwrap(),
            rect(-400, -200, 100, 200)
        );
    }

    #[test]
    fn rejects_empty_tiny_and_pathologically_large_regions() {
        assert!(matches!(
            Rect::from_drag(point(0, 0), point(0, 100)),
            Err(AppError::CaptureImageInvalid)
        ));
        assert!(matches!(
            Rect::from_drag(point(0, 0), point(1, 100)),
            Err(AppError::CaptureImageInvalid)
        ));
        assert!(matches!(
            Rect::from_drag(point(i32::MIN, i32::MIN), point(i32::MAX, i32::MAX)),
            Err(AppError::CaptureTooLarge)
        ));
        assert_eq!(rect(0, 0, 12000, 5000).rgba_len().unwrap(), 240_000_000);
        assert!(matches!(
            rect(0, 0, 12000, 5001).rgba_len(),
            Err(AppError::CaptureTooLarge)
        ));
    }

    #[test]
    fn virtual_desktop_monitor_tiles_preserve_offsets() {
        let selection = rect(-200, -100, 2200, 600);
        let monitors = [
            rect(-1600, 0, 0, 900),
            rect(0, 0, 1920, 1080),
            rect(1920, -800, 3200, 0),
            rect(4000, 0, 5000, 1000),
        ];
        assert_eq!(
            monitor_tiles(selection, &monitors),
            vec![
                MonitorTile {
                    source: rect(-200, 0, 0, 600),
                    offset_x: 0,
                    offset_y: 100,
                },
                MonitorTile {
                    source: rect(0, 0, 1920, 600),
                    offset_x: 200,
                    offset_y: 100,
                },
                MonitorTile {
                    source: rect(1920, -100, 2200, 0),
                    offset_x: 2120,
                    offset_y: 0,
                },
            ]
        );
    }

    #[test]
    fn logical_scaling_is_deterministic_for_supported_dpi_values() {
        for (percent, expected) in [(100, 10), (125, 13), (150, 15), (200, 20)] {
            assert_eq!(logical_to_physical(10, percent).unwrap(), expected);
            assert_eq!(logical_to_physical(-10, percent).unwrap(), -expected);
        }
        assert_eq!(logical_to_physical(1, 150).unwrap(), 2);
        assert_eq!(logical_to_physical(-1, 150).unwrap(), -2);
        assert!(logical_to_physical(100, 0).is_err());
        assert!(logical_to_physical(i32::MAX, 400).is_err());
    }

    #[test]
    fn mixed_dpi_monitors_keep_physical_tile_offsets() {
        // Per-monitor logical offsets scale independently, but the selector
        // and GDI capture use one physical virtual-desktop coordinate space.
        // A negative-origin 125% display sits left of a 200% display.
        let left_local_x = logical_to_physical(80, 125).unwrap();
        let right_local_x = logical_to_physical(100, 200).unwrap();
        assert_eq!(left_local_x, 100);
        assert_eq!(right_local_x, 200);

        let selection = rect(-left_local_x, 50, right_local_x, 250);
        let monitors = [rect(-1920, 0, 0, 1080), rect(0, 0, 2560, 1440)];
        assert_eq!(
            monitor_tiles(selection, &monitors),
            vec![
                MonitorTile {
                    source: rect(-100, 50, 0, 250),
                    offset_x: 0,
                    offset_y: 0,
                },
                MonitorTile {
                    source: rect(0, 50, 200, 250),
                    offset_x: 100,
                    offset_y: 0,
                },
            ]
        );
        // Neither tile receives a second DPI scaling after it has been
        // converted to physical coordinates.
        assert_eq!(selection.width(), 300);
        assert_eq!(selection.height(), 200);
    }

    #[test]
    fn selection_and_monitor_edges_are_half_open() {
        assert_eq!(rect(0, 0, 100, 100).intersect(rect(100, 0, 200, 100)), None);
        assert_eq!(
            rect(0, 0, 100, 100).intersect(rect(99, 99, 200, 200)),
            Some(rect(99, 99, 100, 100))
        );
    }
}
