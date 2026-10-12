use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const TRUSTED_EXECUTABLE_DIRECTORIES: [&str; 4] =
    ["/usr/bin", "/bin", "/usr/local/bin", "/snap/bin"];

#[derive(Clone, Debug)]
pub struct LinuxEnvironmentInfo {
    pub os_release: String,
    pub desktop_environment: String,
    pub session_type: String,
    pub gnome_screenshot: Option<PathBuf>,
    pub tesseract: Option<PathBuf>,
}

pub struct LinuxEnvironmentProbe;

impl LinuxEnvironmentProbe {
    pub fn probe() -> LinuxEnvironmentInfo {
        let os_release = read_os_release();
        // XDG variables are user-controlled. Do not echo arbitrary values into
        // the public diagnostics panel (which must not contain private text).
        let desktop_environment =
            safe_desktop_environment(env::var("XDG_CURRENT_DESKTOP").ok().as_deref()).into();
        let session_type = safe_session_type(env::var("XDG_SESSION_TYPE").ok().as_deref()).into();
        let gnome_screenshot = find_system_executable("gnome-screenshot");
        let tesseract = find_system_executable("tesseract");
        LinuxEnvironmentInfo {
            os_release,
            desktop_environment,
            session_type,
            gnome_screenshot,
            tesseract,
        }
    }
}

fn safe_desktop_environment(raw: Option<&str>) -> &'static str {
    let Some(raw) = raw else { return "unknown" };
    for desktop in raw.split(':') {
        let desktop = desktop.trim();
        if desktop.eq_ignore_ascii_case("gnome") || desktop.eq_ignore_ascii_case("gnome-classic") {
            return "GNOME";
        }
        for (known, label) in [
            ("kde", "KDE Plasma"),
            ("plasma", "KDE Plasma"),
            ("xfce", "XFCE"),
            ("lxqt", "LXQt"),
            ("cinnamon", "Cinnamon"),
            ("x-cinnamon", "Cinnamon"),
            ("mate", "MATE"),
            ("unity", "Unity"),
            ("budgie", "Budgie"),
            ("pantheon", "Pantheon"),
            ("sway", "Sway"),
            ("hyprland", "Hyprland"),
            ("cosmic", "COSMIC"),
        ] {
            if desktop.eq_ignore_ascii_case(known) {
                return label;
            }
        }
    }
    "other"
}

fn safe_session_type(raw: Option<&str>) -> &'static str {
    match raw {
        Some(value) if value.eq_ignore_ascii_case("wayland") => "wayland",
        Some(value) if value.eq_ignore_ascii_case("x11") => "x11",
        Some(value) if value.eq_ignore_ascii_case("tty") => "tty",
        Some(_) => "other",
        None => "unknown",
    }
}

fn read_os_release() -> String {
    fs::read_to_string("/etc/os-release")
        .map(|text| safe_os_release(&text))
        .unwrap_or_else(|_| "Linux (release unknown)".into())
}

// /etc/os-release can contain administrator-provided free-form text. Public
// diagnostics must never echo PRETTY_NAME or arbitrary vendor/version strings.
fn safe_os_release(text: &str) -> String {
    let label = match os_release_field(text, "ID=") {
        Some("ubuntu") => "Ubuntu",
        Some("debian") => "Debian",
        Some("fedora") => "Fedora",
        Some("arch") => "Arch Linux",
        Some("linuxmint") => "Linux Mint",
        Some("rhel") => "Red Hat Enterprise Linux",
        Some("centos") => "CentOS",
        Some("opensuse") | Some("opensuse-leap") => "openSUSE",
        Some("opensuse-tumbleweed") => "openSUSE Tumbleweed",
        Some("alpine") => "Alpine Linux",
        Some("nixos") => "NixOS",
        Some("kali") => "Kali Linux",
        _ => return "Linux (release unknown)".into(),
    };
    match os_release_field(text, "VERSION_ID=") {
        Some(version) if safe_numeric_version(version) => format!("{label} {version}"),
        _ => label.into(),
    }
}

fn os_release_field<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    text.lines()
        .find_map(|line| line.strip_prefix(key))
        .map(|value| value.trim_matches('"'))
}

fn safe_numeric_version(version: &str) -> bool {
    !version.is_empty()
        && version.len() <= 16
        && version.bytes().all(|byte| byte.is_ascii_digit() || byte == b'.')
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
    fn diagnostic_environment_labels_never_echo_untrusted_values() {
        assert_eq!(safe_desktop_environment(Some("ubuntu:GNOME")), "GNOME");
        assert_eq!(safe_desktop_environment(Some("X-Cinnamon")), "Cinnamon");
        assert_eq!(safe_desktop_environment(None), "unknown");
        assert_eq!(
            safe_desktop_environment(Some("private/example/path")),
            "other"
        );
        assert_eq!(safe_session_type(Some("WAYLAND")), "wayland");
        assert_eq!(safe_session_type(Some("X11")), "x11");
        assert_eq!(safe_session_type(Some("private-session")), "other");
        assert_eq!(safe_session_type(None), "unknown");
    }

    #[test]
    fn os_release_diagnostics_use_only_known_labels_and_numeric_versions() {
        assert_eq!(
            safe_os_release("ID=ubuntu\nVERSION_ID=\"24.04\"\nPRETTY_NAME=\"custom machine\"\n"),
            "Ubuntu 24.04"
        );
        assert_eq!(safe_os_release("ID=arch\n"), "Arch Linux");
        assert_eq!(
            safe_os_release("ID=ubuntu\nVERSION_ID=\"24.04/private\"\n"),
            "Ubuntu"
        );
        assert_eq!(
            safe_os_release("ID=custom\nPRETTY_NAME=\"personal build\"\n"),
            "Linux (release unknown)"
        );
    }

    #[test]
    fn missing_system_executable_is_not_fabricated() {
        assert!(find_system_executable("screenshot-ocr-definitely-missing-tool").is_none());
    }
}
