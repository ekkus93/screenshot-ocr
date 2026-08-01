use crate::error::AppError;
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct EnvironmentInfo {
    pub os_release: String,
    pub desktop_environment: String,
    pub session_type: String,
    pub gnome_screenshot: Option<PathBuf>,
    pub tesseract: Option<PathBuf>,
    pub portal_summary: String,
}

pub struct EnvironmentProbe;

impl EnvironmentProbe {
    pub fn probe() -> Result<EnvironmentInfo, AppError> {
        if env::consts::OS != "linux" {
            return Err(AppError::UnsupportedEnvironment);
        }
        let os_release = read_os_release();
        let desktop_environment =
            env::var("XDG_CURRENT_DESKTOP").unwrap_or_else(|_| "unknown".into());
        let session_type = env::var("XDG_SESSION_TYPE").unwrap_or_else(|_| "unknown".into());
        let gnome_screenshot = find_executable("gnome-screenshot");
        let tesseract = find_executable("tesseract");
        let portal_summary = if env::var_os("DBUS_SESSION_BUS_ADDRESS").is_some() {
            "session bus available; explicit area target not yet proven".into()
        } else {
            "session bus unavailable".into()
        };
        Ok(EnvironmentInfo {
            os_release,
            desktop_environment,
            session_type,
            gnome_screenshot,
            tesseract,
            portal_summary,
        })
    }
}

fn read_os_release() -> String {
    fs::read_to_string("/etc/os-release")
        .ok()
        .and_then(|text| {
            text.lines()
                .find(|line| line.starts_with("PRETTY_NAME="))
                .map(str::to_owned)
        })
        .map(|line| {
            line.trim_start_matches("PRETTY_NAME=")
                .trim_matches('"')
                .to_owned()
        })
        .unwrap_or_else(|| "Linux (release unknown)".into())
}

pub fn find_executable(name: &str) -> Option<PathBuf> {
    let path = env::var_os("PATH")?;
    env::split_paths(&path)
        .map(|directory| directory.join(name))
        .find(|candidate| candidate.is_file())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn executable_discovery_rejects_missing_name() {
        assert!(find_executable("screenshot-ocr-definitely-missing-tool").is_none());
    }
}
