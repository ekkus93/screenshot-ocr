use crate::error::AppError;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const TRUSTED_EXECUTABLE_DIRECTORIES: [&str; 4] =
    ["/usr/bin", "/bin", "/usr/local/bin", "/snap/bin"];

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
        let gnome_screenshot = find_system_executable("gnome-screenshot");
        let tesseract = find_system_executable("tesseract");
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

pub fn find_system_executable(name: &str) -> Option<PathBuf> {
    let directories = TRUSTED_EXECUTABLE_DIRECTORIES.map(Path::new);
    find_executable_in(name, directories)
}

fn find_executable_in<'a>(
    name: &str,
    directories: impl IntoIterator<Item = &'a Path>,
) -> Option<PathBuf> {
    if name.is_empty() || name.contains('/') {
        return None;
    }
    directories
        .into_iter()
        .filter(|directory| directory.is_absolute())
        .map(|directory| directory.join(name))
        .find(|candidate| is_executable_regular_file(candidate))
}

#[cfg(unix)]
fn is_executable_regular_file(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;

    let Ok(metadata) = fs::symlink_metadata(path) else {
        return false;
    };
    metadata.file_type().is_file()
        && !metadata.file_type().is_symlink()
        && metadata.permissions().mode() & 0o111 != 0
}

#[cfg(not(unix))]
fn is_executable_regular_file(path: &Path) -> bool {
    path.is_file()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn executable_discovery_rejects_invalid_names_and_relative_directories() {
        assert!(find_executable_in("../tesseract", [Path::new("/usr/bin")]).is_none());
        assert!(find_executable_in("tesseract", [Path::new("relative/bin")]).is_none());
    }

    #[cfg(unix)]
    #[test]
    fn executable_discovery_requires_executable_regular_file() {
        use std::os::unix::fs::PermissionsExt;

        let directory = tempdir().expect("tempdir");
        let candidate = directory.path().join("tool");
        fs::write(&candidate, b"#!/bin/sh\n").expect("write tool");
        fs::set_permissions(&candidate, fs::Permissions::from_mode(0o600))
            .expect("set non-executable mode");
        assert!(find_executable_in("tool", [directory.path()]).is_none());

        fs::set_permissions(&candidate, fs::Permissions::from_mode(0o700))
            .expect("set executable mode");
        assert_eq!(
            find_executable_in("tool", [directory.path()]),
            Some(candidate)
        );
    }

    #[test]
    fn missing_system_executable_is_not_fabricated() {
        assert!(find_system_executable("screenshot-ocr-definitely-missing-tool").is_none());
    }
}
