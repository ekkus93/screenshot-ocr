//! Trusted Windows OCR runtime lookup; never searches PATH or the CWD.
use crate::error::AppError;
use crate::ocr::OcrRuntime;
use std::path::{Path, PathBuf};

/// Installer resource layout, relative to the application's own executable.
/// Both the binary and English language data are required for an OCR-ready app.
fn resource_root(executable: &Path) -> Result<PathBuf, AppError> {
    let directory = executable.parent().ok_or(AppError::OcrEngineUnavailable)?;
    Ok(directory.join("resources").join("ocr"))
}

fn nonempty_file(path: &Path) -> bool {
    path.metadata()
        .is_ok_and(|metadata| metadata.is_file() && metadata.len() > 0)
}

fn resolve_at(executable: &Path) -> Result<OcrRuntime, AppError> {
    let root = resource_root(executable)?;
    let binary = root.join("tesseract.exe");
    if !nonempty_file(&binary) {
        return Err(AppError::OcrEngineUnavailable);
    }
    let tessdata = root.join("tessdata");
    if !nonempty_file(&tessdata.join("eng.traineddata")) {
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

pub fn packaged_runtime_status() -> String {
    match resolve_packaged_runtime() {
        Ok(_) => "available".into(),
        Err(AppError::OcrLanguageMissing) => "missing English language data".into(),
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
    fn resolver_ignores_untrusted_working_directory_and_path() {
        let root = tempdir().unwrap();
        fs::write(root.path().join("tesseract.exe"), b"fixture").unwrap();
        assert!(matches!(
            resolve_at(&root.path().join("bin").join("screenshot-ocr.exe")),
            Err(AppError::OcrEngineUnavailable)
        ));
    }
}
