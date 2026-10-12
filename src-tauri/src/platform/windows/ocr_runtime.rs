//! Trusted Windows OCR runtime lookup; never searches PATH or the CWD.
use crate::error::AppError;
use crate::ocr::OcrRuntime;
use std::os::windows::fs::MetadataExt;
use std::path::{Path, PathBuf};

// Windows FILE_ATTRIBUTE_REPARSE_POINT. Reject NTFS links and junctions in
// the app-owned OCR subtree before selecting a native executable or data.
const REPARSE_POINT: u32 = 0x400;

/// Installer resource layout, relative to the application's own executable.
/// Both the binary and English language data are required for an OCR-ready app.
fn resource_root(executable: &Path) -> Result<PathBuf, AppError> {
    let directory = executable.parent().ok_or(AppError::OcrEngineUnavailable)?;
    Ok(directory.join("resources").join("ocr"))
}

fn directly_owned_directory(path: &Path) -> bool {
    path.symlink_metadata().is_ok_and(|metadata| {
        metadata.is_dir() && (metadata.file_attributes() & REPARSE_POINT) == 0
    })
}

fn nonempty_file(path: &Path) -> bool {
    path.symlink_metadata().is_ok_and(|metadata| {
        metadata.is_file()
            && metadata.len() > 0
            && (metadata.file_attributes() & REPARSE_POINT) == 0
    })
}

// Fail closed on unexpected or redirected app-owned runtime entries, not
// just malformed adjacent DLLs. The pinned NSIS staging layout consists of
// tesseract.exe, tessdata/eng.traineddata and adjacent DLLs only. In particular,
// an unrelated reparse-pointed file or directory must not be silently ignored.
// This is a bounded layout check, not a native-loader or cryptographic audit.
fn packaged_runtime_tree_is_regular(root: &Path) -> bool {
    let Ok(entries) = std::fs::read_dir(root) else {
        return false;
    };
    let mut saw_binary = false;
    let mut saw_tessdata = false;
    for (index, entry) in entries.enumerate() {
        if index >= 256 {
            return false;
        }
        let Ok(entry) = entry else {
            return false;
        };
        let Some(name) = entry.file_name().to_str().map(str::to_ascii_lowercase) else {
            return false;
        };
        let path = entry.path();
        match name.as_str() {
            "tesseract.exe" => {
                if saw_binary || !nonempty_file(&path) {
                    return false;
                }
                saw_binary = true;
            }
            "tessdata" => {
                if saw_tessdata || !directly_owned_directory(&path) {
                    return false;
                }
                saw_tessdata = true;
            }
            name if name.ends_with(".dll") => {
                if !nonempty_file(&path) {
                    return false;
                }
            }
            _ => return false,
        }
    }
    if !saw_binary || !saw_tessdata {
        return false;
    }
    let Ok(entries) = std::fs::read_dir(root.join("tessdata")) else {
        return false;
    };
    let mut saw_english = false;
    for (index, entry) in entries.enumerate() {
        if index >= 16 {
            return false;
        }
        let Ok(entry) = entry else {
            return false;
        };
        // Missing, empty, or redirected English data is reported separately
        // by resolve_at as OcrLanguageMissing, not a generic engine failure.
        if entry.file_name().to_str() != Some("eng.traineddata") || saw_english {
            return false;
        }
        saw_english = true;
    }
    true
}

fn resolve_at(executable: &Path) -> Result<OcrRuntime, AppError> {
    // The resource subtree can be regular files even when an earlier
    // executable-directory ancestor is a junction to an unrelated tree.
    // Reject that redirection before constructing any native runtime paths.
    let directory = executable.parent().ok_or(AppError::OcrEngineUnavailable)?;
    if !directory.is_absolute() || !directory.ancestors().all(directly_owned_directory) {
        return Err(AppError::OcrEngineUnavailable);
    }
    let root = resource_root(executable)?;
    // A junction/symlink in either directory can redirect even correctly
    // app-relative file paths outside the installed OCR resource tree.
    let resources = executable
        .parent()
        .ok_or(AppError::OcrEngineUnavailable)?
        .join("resources");
    if !directly_owned_directory(&resources) || !directly_owned_directory(&root) {
        return Err(AppError::OcrEngineUnavailable);
    }
    if !packaged_runtime_tree_is_regular(&root) {
        return Err(AppError::OcrEngineUnavailable);
    }
    let binary = root.join("tesseract.exe");
    if !nonempty_file(&binary) {
        return Err(AppError::OcrEngineUnavailable);
    }
    let tessdata = root.join("tessdata");
    if !directly_owned_directory(&tessdata) || !nonempty_file(&tessdata.join("eng.traineddata")) {
        return Err(AppError::OcrLanguageMissing);
    }
    // The portable constructor is shared by both production adapters.
    // Windows alone selects the app-controlled working directory.
    let mut runtime = OcrRuntime::new(binary, Some(tessdata));
    runtime.working_directory = Some(root);
    Ok(runtime)
}

pub fn resolve_packaged_runtime() -> Result<OcrRuntime, AppError> {
    let executable = std::env::current_exe().map_err(|_| AppError::OcrEngineUnavailable)?;
    resolve_at(&executable)
}

/// Stable, content-free status: never return a path or native error message.
fn runtime_status_at(executable: &Path) -> &'static str {
    match resolve_at(executable) {
        // A file-presence check is not proof that native DLLs load or OCR runs.
        Ok(_) => "packaged files present; OCR probe separate",
        Err(AppError::OcrLanguageMissing) => "missing English language data",
        Err(_) => "unavailable",
    }
}

pub fn packaged_runtime_status() -> String {
    match std::env::current_exe() {
        Ok(executable) => runtime_status_at(&executable).into(),
        Err(_) => "unavailable".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn resolver_requires_app_owned_executable_and_language_data() {
        let root = tempdir().unwrap();
        let executable = root.path().join("screenshot-ocr.exe");
        assert!(matches!(
            resolve_at(&executable),
            Err(AppError::OcrEngineUnavailable)
        ));
        let ocr = root.path().join("resources").join("ocr");
        fs::create_dir_all(ocr.join("tessdata")).unwrap();
        fs::write(ocr.join("tesseract.exe"), b"").unwrap();
        assert!(matches!(
            resolve_at(&executable),
            Err(AppError::OcrEngineUnavailable)
        ));
        fs::write(ocr.join("tesseract.exe"), b"synthetic fixture").unwrap();
        assert!(matches!(
            resolve_at(&executable),
            Err(AppError::OcrLanguageMissing)
        ));
        fs::write(ocr.join("tessdata").join("eng.traineddata"), b"").unwrap();
        assert!(matches!(
            resolve_at(&executable),
            Err(AppError::OcrLanguageMissing)
        ));
        fs::write(ocr.join("tessdata").join("eng.traineddata"), b"fixture").unwrap();
        let runtime = resolve_at(&executable).unwrap();
        assert_eq!(runtime.executable, ocr.join("tesseract.exe"));
        assert_eq!(runtime.tessdata_dir, Some(ocr.join("tessdata")));
        assert_eq!(runtime.working_directory, Some(ocr));
    }

    #[test]
    fn diagnostic_status_distinguishes_missing_runtime_and_english_data() {
        let root = tempdir().unwrap();
        let executable = root.path().join("screenshot-ocr.exe");
        let ocr = root.path().join("resources").join("ocr");
        assert_eq!(runtime_status_at(&executable), "unavailable");

        fs::create_dir_all(ocr.join("tessdata")).unwrap();
        fs::write(ocr.join("tesseract.exe"), b"synthetic fixture").unwrap();
        assert_eq!(
            runtime_status_at(&executable),
            "missing English language data"
        );

        fs::write(ocr.join("tessdata").join("eng.traineddata"), b"fixture").unwrap();
        assert_eq!(
            runtime_status_at(&executable),
            "packaged files present; OCR probe separate"
        );
    }

    #[test]
    fn resolver_ignores_untrusted_working_directory_and_path() {
        let root = tempdir().unwrap();
        fs::write(root.path().join("tesseract.exe"), b"fixture").unwrap();
        assert!(matches!(
            resolve_at(&root.path().join("bin").join("screenshot-ocr.exe")),
            Err(AppError::OcrEngineUnavailable)
        ));
    }

    #[test]
    fn runtime_rejects_file_and_directory_masquerading() {
        let root = tempdir().unwrap();
        let executable = root.path().join("screenshot-ocr.exe");
        let resources = root.path().join("resources");
        // A regular file cannot substitute for the owned resources directory.
        fs::write(&resources, b"synthetic fixture").unwrap();
        assert!(matches!(
            resolve_at(&executable),
            Err(AppError::OcrEngineUnavailable)
        ));
        fs::remove_file(&resources).unwrap();
        let ocr = resources.join("ocr");
        fs::create_dir_all(ocr.join("tessdata")).unwrap();
        fs::write(ocr.join("tesseract.exe"), b"fixture").unwrap();
        fs::write(ocr.join("tessdata").join("eng.traineddata"), b"fixture").unwrap();
        assert!(resolve_at(&executable).is_ok());
        // A directory masquerading as a native executable cannot be launched.
        fs::remove_file(ocr.join("tesseract.exe")).unwrap();
        fs::create_dir(ocr.join("tesseract.exe")).unwrap();
        assert!(matches!(
            resolve_at(&executable),
            Err(AppError::OcrEngineUnavailable)
        ));
    }

    #[test]
    fn resolver_rejects_empty_directory_and_linked_adjacent_dlls() {
        let root = tempdir().unwrap();
        let executable = root.path().join("screenshot-ocr.exe");
        let ocr = root.path().join("resources").join("ocr");
        fs::create_dir_all(ocr.join("tessdata")).unwrap();
        fs::write(ocr.join("tesseract.exe"), b"fixture").unwrap();
        fs::write(ocr.join("tessdata").join("eng.traineddata"), b"fixture").unwrap();
        assert!(resolve_at(&executable).is_ok());

        let library = ocr.join("synthetic.DLL");
        fs::write(&library, b"").unwrap();
        assert!(matches!(
            resolve_at(&executable),
            Err(AppError::OcrEngineUnavailable)
        ));
        fs::write(&library, b"fixture").unwrap();
        assert!(resolve_at(&executable).is_ok());
        fs::remove_file(&library).unwrap();
        fs::create_dir(&library).unwrap();
        assert!(matches!(
            resolve_at(&executable),
            Err(AppError::OcrEngineUnavailable)
        ));
        fs::remove_dir(&library).unwrap();

        let external = root.path().join("external.dll");
        fs::write(&external, b"fixture").unwrap();
        if std::os::windows::fs::symlink_file(&external, &library).is_ok() {
            assert!(matches!(
                resolve_at(&executable),
                Err(AppError::OcrEngineUnavailable)
            ));
        }
    }

    #[test]
    fn resolver_rejects_symlinked_resources_when_windows_allows_creation() {
        let root = tempdir().unwrap();
        let executable = root.path().join("screenshot-ocr.exe");
        let target = root.path().join("external");
        fs::create_dir_all(target.join("ocr").join("tessdata")).unwrap();
        fs::write(target.join("ocr").join("tesseract.exe"), b"fixture").unwrap();
        fs::write(
            target.join("ocr").join("tessdata").join("eng.traineddata"),
            b"fixture",
        )
        .unwrap();
        // Some Windows CI accounts lack symlink privileges: the file-vs-dir
        // and ordinary packaged runtime tests remain mandatory regardless.
        if std::os::windows::fs::symlink_dir(&target, root.path().join("resources")).is_ok() {
            assert!(matches!(
                resolve_at(&executable),
                Err(AppError::OcrEngineUnavailable)
            ));
        }
    }

    #[test]
    fn resolver_rejects_redirected_tessdata_and_executable_when_links_are_supported() {
        let root = tempdir().unwrap();
        let executable = root.path().join("screenshot-ocr.exe");
        let ocr = root.path().join("resources").join("ocr");
        let tessdata = ocr.join("tessdata");
        fs::create_dir_all(&tessdata).unwrap();
        let binary = ocr.join("tesseract.exe");
        fs::write(&binary, b"fixture").unwrap();
        fs::write(tessdata.join("eng.traineddata"), b"fixture").unwrap();
        assert!(resolve_at(&executable).is_ok());

        let external_binary = root.path().join("external.exe");
        fs::write(&external_binary, b"fixture").unwrap();
        fs::remove_file(&binary).unwrap();
        if std::os::windows::fs::symlink_file(&external_binary, &binary).is_ok() {
            assert!(matches!(
                resolve_at(&executable),
                Err(AppError::OcrEngineUnavailable)
            ));
            fs::remove_file(&binary).unwrap();
        }
        fs::write(&binary, b"fixture").unwrap();

        let external_data = root.path().join("external-data");
        fs::create_dir(&external_data).unwrap();
        fs::write(external_data.join("eng.traineddata"), b"fixture").unwrap();
        fs::remove_dir_all(&tessdata).unwrap();
        if std::os::windows::fs::symlink_dir(&external_data, &tessdata).is_ok() {
            assert!(matches!(
                resolve_at(&executable),
                Err(AppError::OcrLanguageMissing)
            ));
        }
    }

    #[test]
    fn resolver_rejects_redirected_executable_parent_when_links_are_supported() {
        let root = tempdir().unwrap();
        let actual = root.path().join("actual");
        let ocr = actual.join("resources").join("ocr");
        fs::create_dir_all(ocr.join("tessdata")).unwrap();
        fs::write(ocr.join("tesseract.exe"), b"fixture").unwrap();
        fs::write(ocr.join("tessdata").join("eng.traineddata"), b"fixture").unwrap();
        assert!(resolve_at(&actual.join("screenshot-ocr.exe")).is_ok());

        let alias = root.path().join("redirected");
        if std::os::windows::fs::symlink_dir(&actual, &alias).is_ok() {
            assert!(matches!(
                resolve_at(&alias.join("screenshot-ocr.exe")),
                Err(AppError::OcrEngineUnavailable)
            ));
        }
    }

    #[test]
    fn resolver_rejects_unreviewed_runtime_files_and_nested_directories() {
        let root = tempdir().unwrap();
        let executable = root.path().join("screenshot-ocr.exe");
        let ocr = root.path().join("resources").join("ocr");
        let tessdata = ocr.join("tessdata");
        fs::create_dir_all(&tessdata).unwrap();
        fs::write(ocr.join("tesseract.exe"), b"fixture").unwrap();
        fs::write(tessdata.join("eng.traineddata"), b"fixture").unwrap();
        fs::write(ocr.join("synthetic.dll"), b"fixture").unwrap();
        assert!(resolve_at(&executable).is_ok());

        let unexpected = ocr.join("unreviewed.txt");
        fs::write(&unexpected, b"fixture").unwrap();
        assert!(matches!(
            resolve_at(&executable),
            Err(AppError::OcrEngineUnavailable)
        ));
        fs::remove_file(&unexpected).unwrap();

        fs::create_dir(&unexpected).unwrap();
        assert!(matches!(
            resolve_at(&executable),
            Err(AppError::OcrEngineUnavailable)
        ));
        fs::remove_dir(&unexpected).unwrap();

        let extra_data = tessdata.join("unreviewed.traineddata");
        fs::write(&extra_data, b"fixture").unwrap();
        assert!(matches!(
            resolve_at(&executable),
            Err(AppError::OcrEngineUnavailable)
        ));
        fs::remove_file(&extra_data).unwrap();
        assert!(resolve_at(&executable).is_ok());
    }

    #[test]
    fn resolver_rejects_unrelated_linked_entries_when_symlinks_are_available() {
        let root = tempdir().unwrap();
        let executable = root.path().join("screenshot-ocr.exe");
        let ocr = root.path().join("resources").join("ocr");
        fs::create_dir_all(ocr.join("tessdata")).unwrap();
        fs::write(ocr.join("tesseract.exe"), b"fixture").unwrap();
        fs::write(ocr.join("tessdata").join("eng.traineddata"), b"fixture").unwrap();
        let external = root.path().join("external.txt");
        fs::write(&external, b"PRIVATE_SYNTHETIC_FIXTURE").unwrap();
        let link = ocr.join("unexpected.txt");
        if std::os::windows::fs::symlink_file(&external, &link).is_ok() {
            assert!(matches!(
                resolve_at(&executable),
                Err(AppError::OcrEngineUnavailable)
            ));
            assert_eq!(fs::read(&external).unwrap(), b"PRIVATE_SYNTHETIC_FIXTURE");
        }
    }

    #[test]
    fn resolver_rejects_unbounded_adjacent_runtime_entries() {
        let root = tempdir().unwrap();
        let executable = root.path().join("screenshot-ocr.exe");
        let ocr = root.path().join("resources").join("ocr");
        fs::create_dir_all(ocr.join("tessdata")).unwrap();
        fs::write(ocr.join("tesseract.exe"), b"fixture").unwrap();
        fs::write(ocr.join("tessdata").join("eng.traineddata"), b"fixture").unwrap();
        assert!(resolve_at(&executable).is_ok());

        // The limit includes the executable and tessdata directory; 257
        // entries must fail closed regardless of filesystem enumeration order.
        for index in 0..255 {
            fs::write(ocr.join(format!("extra-{index:03}.txt")), b"fixture").unwrap();
        }
        assert!(matches!(
            resolve_at(&executable),
            Err(AppError::OcrEngineUnavailable)
        ));
    }
}
