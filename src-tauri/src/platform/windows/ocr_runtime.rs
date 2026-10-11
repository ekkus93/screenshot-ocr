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

fn resolve_at(executable: &Path) -> Result<OcrRuntime, AppError> {
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
    let binary = root.join("tesseract.exe");
    if !nonempty_file(&binary) {
        return Err(AppError::OcrEngineUnavailable);
    }
    let tessdata = root.join("tessdata");
    if !directly_owned_directory(&tessdata) || !nonempty_file(&tessdata.join("eng.traineddata")) {
        return Err(AppError::OcrLanguageMissing);
    }
    Ok(OcrRuntime {
        executable: binary,
        tessdata_dir: Some(tessdata),
        working_directory: Some(root),
    })
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
}
