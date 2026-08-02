use crate::capture::EnvironmentInfo;
use crate::error::ErrorCode;
use serde::Serialize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

#[derive(Debug)]
pub struct RuntimeDiagnostics {
    last_error_code: Mutex<Option<ErrorCode>>,
    cleanup_failure_count: AtomicU64,
    tray_status: Mutex<String>,
    shortcut_status: Mutex<String>,
}

impl Default for RuntimeDiagnostics {
    fn default() -> Self {
        Self {
            last_error_code: Mutex::new(None),
            cleanup_failure_count: AtomicU64::new(0),
            tray_status: Mutex::new("not initialized".into()),
            shortcut_status: Mutex::new("not initialized".into()),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeDiagnosticsSnapshot {
    pub last_error_code: Option<String>,
    pub cleanup_failure_count: u64,
    pub tray_status: String,
    pub shortcut_status: String,
}

impl Default for RuntimeDiagnosticsSnapshot {
    fn default() -> Self {
        Self {
            last_error_code: None,
            cleanup_failure_count: 0,
            tray_status: "not initialized".into(),
            shortcut_status: "not initialized".into(),
        }
    }
}

impl RuntimeDiagnostics {
    pub fn record(&self, code: ErrorCode) {
        if code == ErrorCode::TemporaryCleanupFailed {
            self.cleanup_failure_count.fetch_add(1, Ordering::Relaxed);
        }
        if let Ok(mut last_error_code) = self.last_error_code.lock() {
            *last_error_code = Some(code);
        }
    }

    pub fn set_tray_status(&self, status: impl Into<String>) {
        if let Ok(mut tray_status) = self.tray_status.lock() {
            *tray_status = status.into();
        }
    }

    pub fn set_shortcut_status(&self, status: impl Into<String>) {
        if let Ok(mut shortcut_status) = self.shortcut_status.lock() {
            *shortcut_status = status.into();
        }
    }

    pub fn snapshot(&self) -> RuntimeDiagnosticsSnapshot {
        let last_error_code = self
            .last_error_code
            .lock()
            .ok()
            .and_then(|last_error_code| *last_error_code)
            .map(serialized_error_code);
        RuntimeDiagnosticsSnapshot {
            last_error_code,
            cleanup_failure_count: self.cleanup_failure_count.load(Ordering::Relaxed),
            tray_status: self
                .tray_status
                .lock()
                .map(|status| status.clone())
                .unwrap_or_else(|_| "unavailable".into()),
            shortcut_status: self
                .shortcut_status
                .lock()
                .map(|status| status.clone())
                .unwrap_or_else(|_| "unavailable".into()),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostics {
    pub app_version: String,
    pub os_release: String,
    pub desktop_environment: String,
    pub session_type: String,
    pub portal_summary: String,
    pub gnome_screenshot: String,
    pub tesseract: String,
    pub installed_languages: Vec<String>,
    pub clipboard_status: String,
    pub tray_status: String,
    pub shortcut_status: String,
    pub settings_schema_version: u32,
    pub last_error_code: Option<String>,
    pub cleanup_failure_count: u64,
}

impl Diagnostics {
    pub fn from_environment(
        environment: &EnvironmentInfo,
        languages: Vec<String>,
        portal_summary: String,
        runtime: RuntimeDiagnosticsSnapshot,
    ) -> Self {
        Self {
            app_version: env!("CARGO_PKG_VERSION").into(),
            os_release: environment.os_release.clone(),
            desktop_environment: environment.desktop_environment.clone(),
            session_type: environment.session_type.clone(),
            portal_summary,
            gnome_screenshot: availability(environment.gnome_screenshot.is_some()),
            tesseract: availability(environment.tesseract.is_some()),
            installed_languages: languages,
            clipboard_status: "available through Tauri clipboard manager".into(),
            tray_status: runtime.tray_status,
            shortcut_status: runtime.shortcut_status,
            settings_schema_version: 1,
            last_error_code: runtime.last_error_code,
            cleanup_failure_count: runtime.cleanup_failure_count,
        }
    }
}

fn availability(available: bool) -> String {
    if available {
        "available".into()
    } else {
        "unavailable".into()
    }
}

fn serialized_error_code(code: ErrorCode) -> String {
    serde_json::to_value(code)
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned))
        .unwrap_or_else(|| "internal_error".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn environment() -> EnvironmentInfo {
        EnvironmentInfo {
            os_release: "Ubuntu".into(),
            desktop_environment: "GNOME".into(),
            session_type: "wayland".into(),
            gnome_screenshot: Some(PathBuf::from("/sensitive/path")),
            tesseract: None,
        }
    }

    #[test]
    fn diagnostics_never_include_executable_paths() {
        let json = serde_json::to_string(&Diagnostics::from_environment(
            &environment(),
            vec!["eng".into()],
            "Screenshot v3; area target available".into(),
            RuntimeDiagnosticsSnapshot::default(),
        ))
        .expect("serialize");
        assert!(!json.contains("/sensitive/path"));
    }

    #[test]
    fn runtime_diagnostics_retain_only_codes_counts_and_safe_statuses() {
        let runtime = RuntimeDiagnostics::default();
        runtime.set_tray_status("available");
        runtime.set_shortcut_status("registered");
        runtime.record(ErrorCode::OcrFailed);
        runtime.record(ErrorCode::TemporaryCleanupFailed);
        let snapshot = runtime.snapshot();
        assert_eq!(
            snapshot,
            RuntimeDiagnosticsSnapshot {
                last_error_code: Some("temporary_cleanup_failed".into()),
                cleanup_failure_count: 1,
                tray_status: "available".into(),
                shortcut_status: "registered".into(),
            }
        );
    }

    #[test]
    fn cleanup_failures_are_counted_independently_of_last_error() {
        let runtime = RuntimeDiagnostics::default();
        runtime.record(ErrorCode::TemporaryCleanupFailed);
        runtime.record(ErrorCode::ClipboardWriteFailed);
        let snapshot = runtime.snapshot();
        assert_eq!(snapshot.cleanup_failure_count, 1);
        assert_eq!(
            snapshot.last_error_code,
            Some("clipboard_write_failed".into())
        );
    }
}
