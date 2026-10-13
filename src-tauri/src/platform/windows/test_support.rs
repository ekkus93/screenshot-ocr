//! Windows-only synthetic OCR child-lifetime assertions.
//! Kept in the platform adapter, never in portable OCR policy.

use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
use std::path::Path;
use windows_sys::Win32::Foundation::WAIT_OBJECT_0;
use windows_sys::Win32::System::Threading::{OpenProcess, WaitForSingleObject};

/// Hold the exact process handle before cancelling, so PID recycling cannot
/// make a stale-process assertion pass.
pub(crate) fn fake_process_handle(marker_dir: &Path, mode: &str) -> OwnedHandle {
    let pid = std::fs::read_to_string(marker_dir.join(format!("{mode}.pid")))
        .expect("synthetic Windows child PID marker")
        .trim()
        .parse::<u32>()
        .expect("numeric synthetic child PID");
    // SYNCHRONIZE permits waiting without process mutation rights.
    let handle = unsafe { OpenProcess(0x0010_0000, 0, pid) };
    assert!(
        !handle.is_null(),
        "synthetic Windows child must still be running"
    );
    // SAFETY: OpenProcess returned a distinct non-null owned handle.
    unsafe { OwnedHandle::from_raw_handle(handle as _) }
}

/// Cancellation only passes after the exact spawned child has exited.
pub(crate) fn assert_fake_process_exited(handle: &OwnedHandle) {
    // SAFETY: The owned handle is a live reference to the exact process.
    let result = unsafe { WaitForSingleObject(handle.as_raw_handle() as _, 0) };
    assert_eq!(
        result, WAIT_OBJECT_0,
        "synthetic Windows OCR child still running after cancellation"
    );
}
