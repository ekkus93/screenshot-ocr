use crate::error::AppError;
use crate::models::{CaptureBackendPreference, TextMode};
use crate::platform::PlatformServices;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;

const SETTINGS_FILE: &str = "settings.json";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AppSettings {
    pub schema_version: u32,
    pub language: String,
    pub text_mode: TextMode,
    pub preview_before_copy: bool,
    pub preserve_whitespace: bool,
    pub notify_after_copy: bool,
    pub start_at_login: bool,
    pub close_to_tray: bool,
    pub capture_backend: CaptureBackendPreference,
    pub shortcut: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            schema_version: 1,
            language: "eng".into(),
            text_mode: TextMode::Terminal,
            preview_before_copy: false,
            preserve_whitespace: true,
            notify_after_copy: true,
            start_at_login: false,
            close_to_tray: true,
            capture_backend: CaptureBackendPreference::Auto,
            shortcut: "Super+Shift+O".into(),
        }
    }
}

impl AppSettings {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.schema_version != 1 || self.language != "eng" || self.shortcut != "Super+Shift+O" {
            return Err(AppError::SettingsInvalid);
        }
        if self.text_mode == TextMode::Terminal && !self.preserve_whitespace {
            return Err(AppError::SettingsInvalid);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsRecoveryWarning {
    pub code: String,
    pub message: String,
    pub guidance: String,
    pub recovered_with_defaults: bool,
}

impl SettingsRecoveryWarning {
    fn invalid_settings() -> Self {
        Self {
            code: "settings_invalid_recovered".into(),
            message: "Settings could not be loaded, so safe defaults were used.".into(),
            guidance: "Review the settings and save them to replace the invalid configuration."
                .into(),
            recovered_with_defaults: true,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsLoadResult {
    pub settings: AppSettings,
    pub warning: Option<SettingsRecoveryWarning>,
}

pub struct SettingsStore {
    directory: PathBuf,
    platform: Arc<dyn PlatformServices>,
}

impl SettingsStore {
    pub fn new(directory: PathBuf, platform: Arc<dyn PlatformServices>) -> Self {
        Self {
            directory,
            platform,
        }
    }

    pub fn load(&self) -> Result<AppSettings, AppError> {
        let path = self.directory.join(SETTINGS_FILE);
        if !path.exists() {
            return Ok(AppSettings::default());
        }
        let bytes = fs::read(&path).map_err(|_| AppError::SettingsInvalid)?;
        match serde_json::from_slice::<AppSettings>(&bytes) {
            // A file that parses but fails validation has to be quarantined just like
            // an unparseable one. Otherwise `load_for_frontend` keeps masking it with
            // defaults (so the UI looks healthy) while every capture keeps failing on
            // `load()`, with nothing to make the state self-heal.
            Ok(settings) => match settings.validate() {
                Ok(()) => Ok(settings),
                Err(error) => {
                    self.quarantine(&path)?;
                    Err(error)
                }
            },
            Err(_) => {
                self.quarantine(&path)?;
                Err(AppError::SettingsInvalid)
            }
        }
    }

    pub fn load_for_frontend(&self) -> Result<SettingsLoadResult, AppError> {
        match self.load() {
            Ok(settings) => Ok(SettingsLoadResult {
                settings,
                warning: None,
            }),
            Err(AppError::SettingsInvalid) => Ok(SettingsLoadResult {
                settings: AppSettings::default(),
                warning: Some(SettingsRecoveryWarning::invalid_settings()),
            }),
            Err(error) => Err(error),
        }
    }

    pub fn save(&self, settings: &AppSettings) -> Result<(), AppError> {
        settings.validate()?;
        fs::create_dir_all(&self.directory).map_err(|_| AppError::SettingsWriteFailed)?;
        self.platform.protect_settings_directory(&self.directory)?;
        let destination = self.directory.join(SETTINGS_FILE);
        // A unique, exclusively created file avoids following a pre-existing
        // settings.json.tmp symlink and lets interrupted writes recover without
        // requiring users to delete a stale temporary file.
        let temporary = self
            .directory
            .join(format!("settings-{}.tmp", uuid::Uuid::new_v4()));
        let serialized =
            serde_json::to_vec_pretty(settings).map_err(|_| AppError::SettingsWriteFailed)?;
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(|_| AppError::SettingsWriteFailed)?;
        let result = (|| {
            self.platform.protect_settings_file(&temporary)?;
            file.write_all(&serialized)
                .and_then(|_| file.write_all(b"\n"))
                .and_then(|_| file.sync_all())
                .map_err(|_| AppError::SettingsWriteFailed)?;
            fs::rename(&temporary, &destination).map_err(|_| AppError::SettingsWriteFailed)
        })();
        if result.is_err() {
            // Never leave a partially written settings file behind on failure.
            let _ = fs::remove_file(&temporary);
        }
        result
    }

    pub fn reset(&self) -> Result<AppSettings, AppError> {
        let settings = AppSettings::default();
        self.save(&settings)?;
        Ok(settings)
    }

    fn quarantine(&self, path: &Path) -> Result<(), AppError> {
        let quarantine = self.directory.join("settings.corrupt.json");
        fs::rename(path, quarantine).map_err(|_| AppError::SettingsWriteFailed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn settings_round_trip_atomically() {
        let dir = tempdir().expect("tempdir");
        let store = SettingsStore::new(dir.path().to_path_buf(), crate::platform::current());
        let settings = AppSettings::default();
        store.save(&settings).expect("save");
        assert_eq!(store.load().expect("load"), settings);
        assert!(!dir.path().join("settings.json.tmp").exists());
    }

    #[cfg(unix)]
    #[test]
    fn unix_settings_permissions_remain_private_through_platform_adapter() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempdir().expect("tempdir");
        let store = SettingsStore::new(dir.path().to_path_buf(), crate::platform::current());
        store.save(&AppSettings::default()).expect("save");
        assert_eq!(
            fs::metadata(dir.path()).unwrap().permissions().mode() & 0o777,
            0o700
        );
        assert_eq!(
            fs::metadata(dir.path().join(SETTINGS_FILE))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }

    #[test]
    fn stale_legacy_temporary_file_does_not_block_settings_updates() {
        let dir = tempdir().expect("tempdir");
        let legacy = dir.path().join("settings.json.tmp");
        fs::write(&legacy, b"stale temporary file").expect("write stale temp");
        let store = SettingsStore::new(dir.path().to_path_buf(), crate::platform::current());
        store
            .save(&AppSettings::default())
            .expect("save despite stale temp");
        assert_eq!(store.load().expect("load"), AppSettings::default());
        assert_eq!(
            fs::read(&legacy).expect("read legacy temp"),
            b"stale temporary file"
        );
        assert!(!dir.path().read_dir().unwrap().any(|entry| {
            entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with("settings-")
        }));
    }

    #[cfg(unix)]
    #[test]
    fn stale_legacy_temporary_symlink_is_never_followed() {
        use std::os::unix::fs::symlink;
        let dir = tempdir().expect("tempdir");
        let target = dir.path().join("unrelated-private-file");
        fs::write(&target, b"must not be overwritten").expect("write target");
        let legacy = dir.path().join("settings.json.tmp");
        symlink(&target, &legacy).expect("create synthetic legacy symlink");
        let store = SettingsStore::new(dir.path().to_path_buf(), crate::platform::current());
        store.save(&AppSettings::default()).expect("safe save");
        assert_eq!(
            fs::read(&target).expect("read target"),
            b"must not be overwritten"
        );
        assert!(fs::symlink_metadata(&legacy)
            .unwrap()
            .file_type()
            .is_symlink());
        assert_eq!(store.load().expect("load"), AppSettings::default());
    }

    #[test]
    fn updating_settings_replaces_existing_file_and_reset_replaces_it_again() {
        let dir = tempdir().expect("tempdir");
        let store = SettingsStore::new(dir.path().to_path_buf(), crate::platform::current());
        let original = AppSettings::default();
        store.save(&original).expect("initial save");
        let mut changed = original.clone();
        changed.preview_before_copy = !original.preview_before_copy;
        store.save(&changed).expect("overwrite existing settings");
        assert_eq!(store.load().expect("load replacement"), changed);
        assert!(!dir.path().join("settings.json.tmp").exists());
        assert_eq!(store.reset().expect("reset"), original);
        assert_eq!(store.load().expect("load reset"), original);
        assert!(!dir.path().join("settings.json.tmp").exists());
    }

    #[test]
    fn corrupt_settings_are_quarantined() {
        let dir = tempdir().expect("tempdir");
        fs::write(dir.path().join(SETTINGS_FILE), b"not json").expect("write corrupt settings");
        let store = SettingsStore::new(dir.path().to_path_buf(), crate::platform::current());
        assert!(matches!(store.load(), Err(AppError::SettingsInvalid)));
        assert!(dir.path().join("settings.corrupt.json").exists());
    }

    #[test]
    fn parseable_but_invalid_settings_are_quarantined_and_self_heal() {
        let dir = tempdir().expect("tempdir");
        // Parses cleanly as JSON and as `AppSettings`, but fails `validate()`.
        let invalid = AppSettings {
            schema_version: 2,
            ..AppSettings::default()
        };
        fs::write(
            dir.path().join(SETTINGS_FILE),
            serde_json::to_vec(&invalid).expect("serialize invalid settings"),
        )
        .expect("write invalid settings");
        let store = SettingsStore::new(dir.path().to_path_buf(), crate::platform::current());

        // First read reports the problem and quarantines the file...
        assert!(matches!(store.load(), Err(AppError::SettingsInvalid)));
        assert!(dir.path().join("settings.corrupt.json").exists());
        assert!(!dir.path().join(SETTINGS_FILE).exists());

        // ...so the capture path stops failing forever on the next read, instead of
        // staying broken while `load_for_frontend` masks it with defaults.
        assert_eq!(
            store.load().expect("load after quarantine"),
            AppSettings::default()
        );
    }

    #[test]
    fn frontend_and_capture_load_paths_agree_after_recovery() {
        let dir = tempdir().expect("tempdir");
        let invalid = AppSettings {
            schema_version: 2,
            ..AppSettings::default()
        };
        fs::write(
            dir.path().join(SETTINGS_FILE),
            serde_json::to_vec(&invalid).expect("serialize invalid settings"),
        )
        .expect("write invalid settings");
        let store = SettingsStore::new(dir.path().to_path_buf(), crate::platform::current());

        let recovered = store.load_for_frontend().expect("frontend load");
        assert!(recovered.warning.is_some());
        // The frontend claims defaults are in use; `load()` must now agree.
        assert_eq!(store.load().expect("capture load"), recovered.settings);
    }

    #[test]
    fn corrupt_settings_return_visible_safe_recovery_result() {
        let dir = tempdir().expect("tempdir");
        fs::write(
            dir.path().join(SETTINGS_FILE),
            br#"{"secret":"SYNTHETIC_SECRET_9f33","path":"/tmp/private"}"#,
        )
        .expect("write corrupt settings");
        let store = SettingsStore::new(dir.path().to_path_buf(), crate::platform::current());
        let result = store.load_for_frontend().expect("recovery result");
        assert_eq!(result.settings, AppSettings::default());
        assert!(result.warning.is_some());
        let json = serde_json::to_string(&result).expect("serialize recovery result");
        assert!(!json.contains("SYNTHETIC_SECRET_9f33"));
        assert!(!json.contains("/tmp/private"));
    }

    #[test]
    fn linux_capture_preferences_remain_valid_for_cross_platform_profile_migration() {
        for capture_backend in [
            CaptureBackendPreference::Auto,
            CaptureBackendPreference::Gnome,
            CaptureBackendPreference::Portal,
        ] {
            let settings = AppSettings {
                capture_backend,
                ..AppSettings::default()
            };
            let serialized = serde_json::to_vec(&settings).expect("serialize settings");
            let parsed: AppSettings =
                serde_json::from_slice(&serialized).expect("parse migrated settings");
            assert_eq!(parsed.capture_backend, capture_backend);
            assert!(parsed.validate().is_ok());
        }
    }
}
