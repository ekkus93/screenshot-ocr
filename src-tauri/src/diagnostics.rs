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
        // Native status strings must not introduce unbounded OS error details.
        let status = status.into();
        let safe = match status.as_str() {
            "available" | "unavailable; main window remains usable" | "not initialized" => {
                status.as_str()
            }
            _ => "unavailable",
        };
        if let Ok(mut tray_status) = self.tray_status.lock() {
            *tray_status = safe.into();
        }
    }

    pub fn set_shortcut_status(&self, status: impl Into<String>) {
        let status = status.into();
        let safe = match status.as_str() {
            "registered"
            | "registration failed; use the documented platform shortcut"
            | "not registered; use the documented platform shortcut"
            | "deferred; use the GNOME custom shortcut"
            | "not initialized" => status.as_str(),
            _ => "registration status unavailable",
        };
        if let Ok(mut shortcut_status) = self.shortcut_status.lock() {
            *shortcut_status = safe.into();
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlatformDiagnostics {
    pub platform: String,
    pub os_release: String,
    pub desktop_environment: String,
    pub session_type: String,
    pub capture_summary: String,
    pub ocr_runtime: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostics {
    pub app_version: String,
    pub platform: String,
    pub os_release: String,
    pub desktop_environment: String,
    pub session_type: String,
    pub capture_summary: String,
    pub ocr_runtime: String,
    pub installed_languages: Vec<String>,
    pub clipboard_status: String,
    pub tray_status: String,
    pub shortcut_status: String,
    pub settings_schema_version: u32,
    pub last_error_code: Option<String>,
    pub cleanup_failure_count: u64,
}

impl Diagnostics {
    pub fn from_platform(
        platform: PlatformDiagnostics,
        languages: Vec<String>,
        runtime: RuntimeDiagnosticsSnapshot,
    ) -> Self {
        Self {
            app_version: env!("CARGO_PKG_VERSION").into(),
            platform: platform.platform,
            os_release: platform.os_release,
            desktop_environment: platform.desktop_environment,
            session_type: platform.session_type,
            capture_summary: platform.capture_summary,
            ocr_runtime: platform.ocr_runtime,
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

fn serialized_error_code(code: ErrorCode) -> String {
    serde_json::to_value(code)
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned))
        .unwrap_or_else(|| "internal_error".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostics_use_only_safe_platform_summary_values() {
        let json = serde_json::to_string(&Diagnostics::from_platform(
            PlatformDiagnostics {
                platform: "linux".into(),
                os_release: "Ubuntu".into(),
                desktop_environment: "GNOME".into(),
                session_type: "wayland".into(),
                capture_summary: "portal available; helper available".into(),
                ocr_runtime: "available".into(),
            },
            vec!["eng".into()],
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
    fn native_statuses_reject_unreviewed_platform_details() {
        let runtime = RuntimeDiagnostics::default();
        runtime.set_tray_status("synthetic-raw-path");
        runtime.set_shortcut_status("synthetic-raw-error");
        let snapshot = runtime.snapshot();
        assert_eq!(snapshot.tray_status, "unavailable");
        assert_eq!(snapshot.shortcut_status, "registration status unavailable");
    }

    #[test]
    fn known_linux_and_windows_desktop_statuses_are_preserved() {
        let runtime = RuntimeDiagnostics::default();
        runtime.set_tray_status("unavailable; main window remains usable");
        runtime.set_shortcut_status("deferred; use the GNOME custom shortcut");
        assert_eq!(
            runtime.snapshot().shortcut_status,
            "deferred; use the GNOME custom shortcut"
        );
        assert_eq!(
            runtime.snapshot().tray_status,
            "unavailable; main window remains usable"
        );
        runtime.set_shortcut_status("registration failed; use the documented platform shortcut");
        assert_eq!(
            runtime.snapshot().shortcut_status,
            "registration failed; use the documented platform shortcut"
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
