use crate::capture::EnvironmentInfo;
use serde::Serialize;

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
    pub settings_schema_version: u32,
    pub last_error_code: Option<String>,
    pub cleanup_failure_count: u64,
}

impl Diagnostics {
    pub fn from_environment(environment: &EnvironmentInfo, languages: Vec<String>) -> Self {
        Self {
            app_version: env!("CARGO_PKG_VERSION").into(),
            os_release: environment.os_release.clone(),
            desktop_environment: environment.desktop_environment.clone(),
            session_type: environment.session_type.clone(),
            portal_summary: environment.portal_summary.clone(),
            gnome_screenshot: availability(environment.gnome_screenshot.is_some()),
            tesseract: availability(environment.tesseract.is_some()),
            installed_languages: languages,
            clipboard_status: "available through Tauri clipboard manager".into(),
            tray_status: "not enabled in the current pre-release build".into(),
            settings_schema_version: 1,
            last_error_code: None,
            cleanup_failure_count: 0,
        }
    }
}

fn availability(available: bool) -> String {
    if available { "available".into() } else { "unavailable".into() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn diagnostics_never_include_executable_paths() {
        let environment = EnvironmentInfo { os_release: "Ubuntu".into(), desktop_environment: "GNOME".into(), session_type: "wayland".into(), gnome_screenshot: Some(PathBuf::from("/sensitive/path")), tesseract: None, portal_summary: "safe".into() };
        let json = serde_json::to_string(&Diagnostics::from_environment(&environment, vec!["eng".into()])).expect("serialize");
        assert!(!json.contains("/sensitive/path"));
    }
}
