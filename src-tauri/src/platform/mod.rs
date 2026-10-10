mod services;

use std::sync::Arc;

pub use services::PlatformServices;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(not(any(target_os = "linux", target_os = "windows")))]
mod unsupported;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
pub fn current() -> Arc<dyn PlatformServices> {
    Arc::new(linux::LinuxPlatform)
}

#[cfg(target_os = "windows")]
pub fn current() -> Arc<dyn PlatformServices> {
    Arc::new(windows::WindowsPlatform)
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
pub fn current() -> Arc<dyn PlatformServices> {
    Arc::new(unsupported::UnsupportedPlatform)
}
