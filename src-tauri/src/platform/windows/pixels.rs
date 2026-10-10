//! Bounded GDI capture. All native resources are owned and released here.
use super::composite;
use super::geometry::{monitor_tiles, Rect};
use super::monitors;
use super::selector;
use crate::cancellation::CancellationToken;
use crate::capture::CaptureBackend;
use crate::error::AppError;
use crate::models::{CaptureBackendId, CapturedImage};
use async_trait::async_trait;
use image::{DynamicImage, RgbaImage};
use std::mem::{size_of, zeroed};
use std::ptr::null_mut;
use windows_sys::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC, GetDIBits,
    ReleaseDC, SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, CAPTUREBLT, DIB_RGB_COLORS,
    HBITMAP, HDC, HGDIOBJ, SRCCOPY,
};

pub struct NativeCapture;

#[async_trait]
impl CaptureBackend for NativeCapture {
    async fn capture_region(
        &self,
        cancellation: &CancellationToken,
    ) -> Result<CapturedImage, AppError> {
        let cancellation = cancellation.clone();
        tokio::task::spawn_blocking(move || {
            let rectangle = selector::select_region(&cancellation)?;
            cancellation.check()?;
            let image = capture_pixels(rectangle, &cancellation)?;
            cancellation.check()?;
            Ok(CapturedImage {
                image: DynamicImage::ImageRgba8(image),
                backend: CaptureBackendId::WindowsNative,
                cleanup_failed: false,
            })
        })
        .await
        .map_err(|_| AppError::CaptureFailed)?
    }
}

struct GdiHandles {
    screen: HDC,
    memory: HDC,
    bitmap: HBITMAP,
    previous: HGDIOBJ,
}

impl GdiHandles {
    fn new(width: i32, height: i32) -> Result<Self, AppError> {
        let mut handles = Self {
            screen: null_mut(),
            memory: null_mut(),
            bitmap: null_mut(),
            previous: null_mut(),
        };
        unsafe {
            handles.screen = GetDC(null_mut());
            if handles.screen.is_null() {
                return Err(AppError::CaptureFailed);
            }
            handles.memory = CreateCompatibleDC(handles.screen);
            if handles.memory.is_null() {
                return Err(AppError::CaptureFailed);
            }
            handles.bitmap = CreateCompatibleBitmap(handles.screen, width, height);
            if handles.bitmap.is_null() {
                return Err(AppError::CaptureFailed);
            }
            handles.previous = SelectObject(handles.memory, handles.bitmap as HGDIOBJ);
            if handles.previous.is_null() {
                return Err(AppError::CaptureFailed);
            }
        }
        Ok(handles)
    }
}

impl Drop for GdiHandles {
    fn drop(&mut self) {
        unsafe {
            if !self.previous.is_null() && !self.memory.is_null() {
                SelectObject(self.memory, self.previous);
            }
            if !self.bitmap.is_null() {
                DeleteObject(self.bitmap as HGDIOBJ);
            }
            if !self.memory.is_null() {
                DeleteDC(self.memory);
            }
            if !self.screen.is_null() {
                ReleaseDC(null_mut(), self.screen);
            }
        }
    }
}

fn capture_tile(rectangle: Rect) -> Result<RgbaImage, AppError> {
    let byte_len = rectangle.rgba_len()?;
    let width = i32::try_from(rectangle.width()).map_err(|_| AppError::CaptureTooLarge)?;
    let height = i32::try_from(rectangle.height()).map_err(|_| AppError::CaptureTooLarge)?;
    let mut handles = GdiHandles::new(width, height)?;
    unsafe {
        if BitBlt(
            handles.memory,
            0,
            0,
            width,
            height,
            handles.screen,
            rectangle.left,
            rectangle.top,
            SRCCOPY | CAPTUREBLT,
        ) == 0
        {
            return Err(AppError::CaptureFailed);
        }
        // GetDIBits requires the bitmap to be deselected from the memory DC.
        if SelectObject(handles.memory, handles.previous).is_null() {
            return Err(AppError::CaptureFailed);
        }
        handles.previous = null_mut();
        let mut info: BITMAPINFO = zeroed();
        info.bmiHeader = BITMAPINFOHEADER {
            biSize: size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width,
            biHeight: -height,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB,
            ..zeroed()
        };
        let mut pixels = vec![0u8; byte_len];
        if GetDIBits(
            handles.screen,
            handles.bitmap,
            0,
            height as u32,
            pixels.as_mut_ptr().cast(),
            &mut info,
            DIB_RGB_COLORS,
        ) != height
        {
            return Err(AppError::CaptureFailed);
        }
        for pixel in pixels.chunks_exact_mut(4) {
            pixel.swap(0, 2);
            pixel[3] = 255;
        }
        RgbaImage::from_raw(width as u32, height as u32, pixels)
            .ok_or(AppError::CaptureImageInvalid)
    }
}

fn capture_pixels(
    selection: Rect,
    cancellation: &CancellationToken,
) -> Result<RgbaImage, AppError> {
    selection.rgba_len()?;
    let displays = monitors::enumerate()?;
    let tiles = monitor_tiles(selection, &displays);
    if tiles.is_empty() {
        return Err(AppError::CaptureImageInvalid);
    }
    let mut captured = Vec::with_capacity(tiles.len());
    for tile in tiles {
        cancellation.check()?;
        captured.push((tile, capture_tile(tile.source)?));
    }
    cancellation.check()?;
    composite::compose(selection, captured)
}
