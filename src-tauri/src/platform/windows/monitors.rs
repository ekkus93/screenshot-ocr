//! Enumerate physical display rectangles for per-monitor GDI capture.
use super::geometry::Rect;
use crate::error::AppError;
use std::ptr::{null, null_mut};
use windows_sys::Win32::Foundation::{LPARAM, RECT};
use windows_sys::Win32::Graphics::Gdi::{EnumDisplayMonitors, HDC, HMONITOR};

unsafe extern "system" fn callback(
    _monitor: HMONITOR,
    _dc: HDC,
    rectangle: *mut RECT,
    context: LPARAM,
) -> i32 {
    if rectangle.is_null() || context == 0 {
        return 0;
    }
    let monitors = &mut *(context as *mut Vec<Rect>);
    if monitors.len() >= 64 {
        return 0;
    }
    let bounds = *rectangle;
    if bounds.right > bounds.left && bounds.bottom > bounds.top {
        monitors.push(Rect {
            left: bounds.left,
            top: bounds.top,
            right: bounds.right,
            bottom: bounds.bottom,
        });
    }
    1
}

pub fn enumerate() -> Result<Vec<Rect>, AppError> {
    let mut monitors = Vec::new();
    let ok = unsafe {
        EnumDisplayMonitors(
            null_mut(),
            null(),
            Some(callback),
            &mut monitors as *mut Vec<Rect> as LPARAM,
        )
    };
    if ok == 0 || monitors.is_empty() {
        return Err(AppError::CaptureBackendUnavailable);
    }
    Ok(monitors)
}
