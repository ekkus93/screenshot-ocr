//! Application-owned Win32 region selector. No shell, disk, or external picker.
use super::geometry::{Point, Rect};
use crate::cancellation::CancellationToken;
use crate::error::AppError;
use std::mem::zeroed;
use std::ptr::{null, null_mut};
use std::thread;
use std::time::Duration;
use windows_sys::Win32::Foundation::{GetLastError, HWND, RECT};
use windows_sys::Win32::Graphics::Gdi::{
    BeginPaint, EndPaint, FillRect, FrameRect, GetStockObject, InvalidateRect, BLACK_BRUSH, HBRUSH,
    PAINTSTRUCT, WHITE_BRUSH,
};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::HiDpi::{
    SetThreadDpiAwarenessContext, DPI_AWARENESS_CONTEXT, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetCapture, ReleaseCapture, SetCapture};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetClientRect, GetCursorPos,
    GetSystemMetrics, GetWindowLongPtrW, LoadCursorW, PeekMessageW, RegisterClassW,
    SetForegroundWindow, SetLayeredWindowAttributes, SetWindowLongPtrW, ShowWindow,
    TranslateMessage, GWLP_USERDATA, IDC_CROSS, LWA_ALPHA, MSG, PM_REMOVE, SM_CXVIRTUALSCREEN,
    SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN, SW_SHOW, WM_CAPTURECHANGED, WM_CLOSE,
    WM_KEYDOWN, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE, WM_PAINT, WM_QUIT, WNDCLASSW,
    WS_EX_LAYERED, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
};

/// The dedicated native selector worker must use physical virtual-screen pixels.
/// Restore the original DPI context before the worker thread returns.
struct SelectorDpiContext(DPI_AWARENESS_CONTEXT);

impl SelectorDpiContext {
    fn enter() -> Result<Self, AppError> {
        // SAFETY: The context switch only affects the current selector worker thread.
        let previous =
            unsafe { SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) };
        if previous.is_null() {
            Err(AppError::CaptureBackendUnavailable)
        } else {
            Ok(Self(previous))
        }
    }
}

impl Drop for SelectorDpiContext {
    fn drop(&mut self) {
        // SAFETY: The guard is not moved between threads and owns this context.
        unsafe { SetThreadDpiAwarenessContext(self.0) };
    }
}

struct Selection {
    origin: Point,
    start: Option<Point>,
    current: Point,
    result: Option<Result<Rect, AppError>>,
}

impl Selection {
    fn begin(&mut self, point: Point) -> bool {
        if self.start.is_some() || self.result.is_some() {
            return false;
        }
        self.start = Some(point);
        self.current = point;
        true
    }

    fn move_to(&mut self, point: Point) -> bool {
        if self.start.is_none() || self.result.is_some() {
            return false;
        }
        self.current = point;
        true
    }

    fn complete(&mut self, end: Result<Point, AppError>) {
        if let Some(start) = self.start.take() {
            if self.result.is_none() {
                self.result = Some(end.and_then(|point| Rect::from_drag(start, point)));
            }
        }
    }

    fn cancel(&mut self) {
        self.start = None;
        if self.result.is_none() {
            self.result = Some(Err(AppError::CaptureCancelled));
        }
    }

    fn capture_lost(&mut self) {
        if self.start.is_some() && self.result.is_none() {
            self.cancel();
        }
    }
}

unsafe fn state(hwnd: HWND) -> Option<&'static mut Selection> {
    let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut Selection;
    ptr.as_mut()
}

unsafe fn cursor_point() -> Result<Point, AppError> {
    let mut cursor = zeroed();
    if GetCursorPos(&mut cursor) == 0 {
        return Err(AppError::CaptureFailed);
    }
    Ok(Point {
        x: cursor.x,
        y: cursor.y,
    })
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    message: u32,
    wparam: usize,
    lparam: isize,
) -> isize {
    match message {
        WM_LBUTTONDOWN => {
            if let Some(selection) = state(hwnd) {
                match cursor_point() {
                    Ok(point) => {
                        if selection.begin(point) {
                            SetCapture(hwnd);
                            // Without mouse ownership, release outside the
                            // overlay may never arrive. Fail rather than
                            // leave an orphaned selector waiting for input.
                            if GetCapture() != hwnd {
                                selection.result = Some(Err(AppError::CaptureFailed));
                            }
                        }
                    }
                    Err(error) => selection.result = Some(Err(error)),
                }
            }
            0
        }
        WM_MOUSEMOVE => {
            if let Some(selection) = state(hwnd) {
                if selection.start.is_some() {
                    if let Ok(point) = cursor_point() {
                        if selection.move_to(point) {
                            InvalidateRect(hwnd, null(), 0);
                        }
                    }
                }
            }
            0
        }
        WM_LBUTTONUP => {
            if let Some(selection) = state(hwnd) {
                if selection.start.is_some() {
                    selection.complete(cursor_point());
                    ReleaseCapture();
                }
            }
            0
        }
        WM_CAPTURECHANGED => {
            if let Some(selection) = state(hwnd) {
                selection.capture_lost();
            }
            0
        }
        WM_KEYDOWN if wparam == 0x1b => {
            if let Some(selection) = state(hwnd) {
                selection.cancel();
            }
            0
        }
        WM_CLOSE => {
            if let Some(selection) = state(hwnd) {
                selection.cancel();
            }
            0
        }
        WM_PAINT => {
            let mut paint: PAINTSTRUCT = zeroed();
            let dc = BeginPaint(hwnd, &mut paint);
            if !dc.is_null() {
                let mut client: RECT = zeroed();
                if GetClientRect(hwnd, &mut client) != 0 {
                    FillRect(dc, &client, GetStockObject(BLACK_BRUSH) as HBRUSH);
                }
                if let Some(selection) = state(hwnd) {
                    if let Some(start) = selection.start {
                        let end = selection.current;
                        let border = RECT {
                            left: start.x.min(end.x).saturating_sub(selection.origin.x),
                            top: start.y.min(end.y).saturating_sub(selection.origin.y),
                            right: start.x.max(end.x).saturating_sub(selection.origin.x),
                            bottom: start.y.max(end.y).saturating_sub(selection.origin.y),
                        };
                        FrameRect(dc, &border, GetStockObject(WHITE_BRUSH) as HBRUSH);
                    }
                }
            }
            EndPaint(hwnd, &paint);
            0
        }
        _ => DefWindowProcW(hwnd, message, wparam, lparam),
    }
}

struct Overlay(HWND);

impl Drop for Overlay {
    fn drop(&mut self) {
        unsafe {
            if !self.0.is_null() {
                SetWindowLongPtrW(self.0, GWLP_USERDATA, 0);
                DestroyWindow(self.0);
            }
        }
    }
}

/// Runs on a dedicated blocking thread: the Win32 message queue belongs to
/// that thread and is pumped frequently enough to observe shared cancellation.
pub fn select_region(cancellation: &CancellationToken) -> Result<Rect, AppError> {
    cancellation.check()?;
    let _dpi_context = SelectorDpiContext::enter()?;
    let mut selection = Selection {
        origin: Point { x: 0, y: 0 },
        start: None,
        current: Point { x: 0, y: 0 },
        result: None,
    };
    let rectangle = unsafe {
        let x = GetSystemMetrics(SM_XVIRTUALSCREEN);
        let y = GetSystemMetrics(SM_YVIRTUALSCREEN);
        let width = GetSystemMetrics(SM_CXVIRTUALSCREEN);
        let height = GetSystemMetrics(SM_CYVIRTUALSCREEN);
        if width <= 0 || height <= 0 {
            return Err(AppError::CaptureBackendUnavailable);
        }
        selection.origin = Point { x, y };
        let class: Vec<u16> = "ScreenshotOcrRegionSelector\0".encode_utf16().collect();
        let instance = GetModuleHandleW(null());
        if instance.is_null() {
            return Err(AppError::CaptureFailed);
        }
        let window_class = WNDCLASSW {
            style: 0,
            lpfnWndProc: Some(window_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: instance,
            hIcon: null_mut(),
            hCursor: LoadCursorW(null_mut(), IDC_CROSS),
            hbrBackground: GetStockObject(BLACK_BRUSH) as HBRUSH,
            lpszMenuName: null(),
            lpszClassName: class.as_ptr(),
        };
        if RegisterClassW(&window_class) == 0 && GetLastError() != 1410 {
            return Err(AppError::CaptureFailed);
        }
        let hwnd = CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_LAYERED,
            class.as_ptr(),
            null(),
            WS_POPUP,
            x,
            y,
            width,
            height,
            null_mut(),
            null_mut(),
            instance,
            null(),
        );
        if hwnd.is_null() {
            return Err(AppError::CaptureFailed);
        }
        let overlay = Overlay(hwnd);
        SetWindowLongPtrW(
            hwnd,
            GWLP_USERDATA,
            &mut selection as *mut Selection as isize,
        );
        if SetLayeredWindowAttributes(hwnd, 0, 95, LWA_ALPHA) == 0 {
            return Err(AppError::CaptureFailed);
        }
        ShowWindow(hwnd, SW_SHOW);
        SetForegroundWindow(hwnd);
        let result = loop {
            if cancellation.is_cancelled() {
                break Err(AppError::CaptureCancelled);
            }
            if let Some(result) = selection.result.take() {
                break result;
            }
            let mut message: MSG = zeroed();
            while PeekMessageW(&mut message, null_mut(), 0, 0, PM_REMOVE) != 0 {
                if message.message == WM_QUIT {
                    selection.result = Some(Err(AppError::CaptureCancelled));
                    break;
                }
                TranslateMessage(&message);
                DispatchMessageW(&message);
            }
            thread::sleep(Duration::from_millis(15));
        };
        drop(overlay);
        result
    };
    cancellation.check()?;
    // Give the desktop compositor a frame to remove the overlay before BitBlt.
    thread::sleep(Duration::from_millis(60));
    cancellation.check()?;
    rectangle
}

#[cfg(test)]
mod tests {
    use super::*;

    fn point(x: i32, y: i32) -> Point {
        Point { x, y }
    }

    fn selection() -> Selection {
        Selection {
            origin: point(-1920, -200),
            start: None,
            current: point(0, 0),
            result: None,
        }
    }

    #[test]
    fn drag_is_normalized_and_duplicate_down_does_not_reset_origin() {
        let mut state = selection();
        assert!(state.begin(point(10, 20)));
        assert!(!state.begin(point(-400, -400)));
        assert!(state.move_to(point(-30, -40)));
        state.complete(Ok(point(-30, -40)));
        assert_eq!(
            state.result.take().unwrap().unwrap(),
            Rect {
                left: -30,
                top: -40,
                right: 10,
                bottom: 20,
            }
        );
        assert!(!state.move_to(point(100, 100)));
    }

    #[test]
    fn lost_mouse_capture_cancels_and_late_release_cannot_succeed() {
        let mut state = selection();
        assert!(state.begin(point(1, 1)));
        state.capture_lost();
        state.complete(Ok(point(100, 100)));
        assert!(matches!(
            state.result,
            Some(Err(AppError::CaptureCancelled))
        ));
    }

    #[test]
    fn escape_cancel_is_idempotent_and_rejects_new_drag() {
        let mut state = selection();
        assert!(state.begin(point(1, 1)));
        state.cancel();
        state.cancel();
        assert!(!state.begin(point(2, 2)));
        assert!(matches!(
            state.result,
            Some(Err(AppError::CaptureCancelled))
        ));
    }

    #[test]
    fn zero_size_and_native_cursor_failure_are_safe() {
        let mut state = selection();
        assert!(state.begin(point(4, 4)));
        state.complete(Ok(point(4, 4)));
        assert!(matches!(
            state.result,
            Some(Err(AppError::CaptureImageInvalid))
        ));
        let mut state = selection();
        assert!(state.begin(point(4, 4)));
        state.complete(Err(AppError::CaptureFailed));
        assert!(matches!(state.result, Some(Err(AppError::CaptureFailed))));
    }
}
