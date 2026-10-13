//! Trusted Windows OCR runtime lookup; never searches PATH or the CWD.
use crate::error::AppError;
use crate::ocr::OcrRuntime;
use std::collections::HashSet;
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

// Only the pinned Mannheim staging inventory is accepted. This allowlist
// prevents unreviewed adjacent DLL names from entering the executable's
// Windows DLL search directory; it is NOT a signature/hash or loader audit.
const REVIEWED_DLL_NAMES: &[&str] = &[
    "libarchive-13.dll",
    "libb2-1.dll",
    "libbrotlicommon.dll",
    "libbrotlidec.dll",
    "libbz2-1.dll",
    "libcairo-2.dll",
    "libcrypto-3-x64.dll",
    "libdatrie-1.dll",
    "libdeflate.dll",
    "libexpat-1.dll",
    "libffi-8.dll",
    "libfontconfig-1.dll",
    "libfreetype-6.dll",
    "libfribidi-0.dll",
    "libgcc_s_seh-1.dll",
    "libgif-7.dll",
    "libgio-2.0-0.dll",
    "libglib-2.0-0.dll",
    "libgmodule-2.0-0.dll",
    "libgobject-2.0-0.dll",
    "libgraphite2.dll",
    "libharfbuzz-0.dll",
    "libiconv-2.dll",
    "libicudt75.dll",
    "libicuin75.dll",
    "libicuuc75.dll",
    "libintl-8.dll",
    "libjbig-0.dll",
    "libjpeg-8.dll",
    "libleptonica-6.dll",
    "liblerc.dll",
    "liblz4.dll",
    "liblzma-5.dll",
    "libopenjp2-7.dll",
    "libpango-1.0-0.dll",
    "libpangocairo-1.0-0.dll",
    "libpangoft2-1.0-0.dll",
    "libpangowin32-1.0-0.dll",
    "libpcre2-8-0.dll",
    "libpixman-1-0.dll",
    "libpng16-16.dll",
    "libsharpyuv-0.dll",
    "libstdc++-6.dll",
    "libtesseract-5.dll",
    "libthai-0.dll",
    "libtiff-6.dll",
    "libwebp-7.dll",
    "libwebpmux-3.dll",
    "libwinpthread-1.dll",
    "libzstd.dll",
    "zlib1.dll",
];

// Windows normally prevents two files whose names differ only by case, but
// case-sensitive NTFS directories can contain both. Reject ambiguous native
// DLL basenames rather than relying on the filesystem's case policy.
fn reviewed_dll_basename_once(name: &str, seen: &mut HashSet<String>) -> bool {
    let canonical = name.to_ascii_lowercase();
    REVIEWED_DLL_NAMES.contains(&canonical.as_str()) && seen.insert(canonical)
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
    let mut seen_dlls = HashSet::new();
    for (index, entry) in entries.enumerate() {
        if index >= 256 {
            return false;
        }
        let Ok(entry) = entry else {
            return false;
        };
        let file_name = entry.file_name();
        let Some(original_name) = file_name.to_str() else {
            return false;
        };
        let name = original_name.to_ascii_lowercase();
        let path = entry.path();
        match name.as_str() {
            "tesseract.exe" => {
                // The runtime constructs this exact app-relative path.
                // Case-sensitive NTFS directories must not silently accept
                // a different spelling that fails at process launch.
                if original_name != "tesseract.exe" || saw_binary || !nonempty_file(&path) {
                    return false;
                }
                saw_binary = true;
            }
            "tessdata" => {
                if original_name != "tessdata" || saw_tessdata {
                    return false;
                }
                saw_tessdata = true;
            }
            name if name.ends_with(".dll") => {
                if !reviewed_dll_basename_once(name, &mut seen_dlls) || !nonempty_file(&path) {
                    return false;
                }
            }
            _ => return false,
        }
    }
    if !saw_binary || !saw_tessdata {
        return false;
    }
    // Preserve the public missing-language error and never enumerate through
    // a reparse-pointed tessdata directory. resolve_at checks this directory
    // immediately afterward and returns OcrLanguageMissing.
    if !directly_owned_directory(&root.join("tessdata")) {
        return true;
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
    fn reviewed_dll_basenames_are_unique_without_case_assumptions() {
        let mut seen = HashSet::new();
        assert!(reviewed_dll_basename_once("libarchive-13.dll", &mut seen));
        assert!(!reviewed_dll_basename_once("LIBARCHIVE-13.DLL", &mut seen));
        assert!(!reviewed_dll_basename_once("libarchive-13.dll", &mut seen));
        assert!(!reviewed_dll_basename_once("unreviewed.dll", &mut seen));
        assert!(reviewed_dll_basename_once("libtesseract-5.dll", &mut seen));
    }

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

        let library = ocr.join("libarchive-13.DLL");
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
    fn resolver_requires_exact_app_resource_names_even_on_case_insensitive_ntfs() {
        let root = tempdir().unwrap();
        let executable = root.path().join("screenshot-ocr.exe");
        let ocr = root.path().join("resources").join("ocr");
        fs::create_dir_all(ocr.join("tessdata")).unwrap();
        fs::write(ocr.join("tesseract.exe"), b"fixture").unwrap();
        fs::write(ocr.join("tessdata").join("eng.traineddata"), b"fixture").unwrap();
        assert!(resolve_at(&executable).is_ok());

        fs::remove_file(ocr.join("tesseract.exe")).unwrap();
        fs::write(ocr.join("TESSERACT.EXE"), b"fixture").unwrap();
        assert!(matches!(
            resolve_at(&executable),
            Err(AppError::OcrEngineUnavailable)
        ));
        fs::remove_file(ocr.join("TESSERACT.EXE")).unwrap();
        fs::write(ocr.join("tesseract.exe"), b"fixture").unwrap();

        fs::remove_dir_all(ocr.join("tessdata")).unwrap();
        fs::create_dir(ocr.join("TESSDATA")).unwrap();
        fs::write(ocr.join("TESSDATA").join("eng.traineddata"), b"fixture").unwrap();
        assert!(matches!(
            resolve_at(&executable),
            Err(AppError::OcrEngineUnavailable)
        ));
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
        fs::write(ocr.join("libarchive-13.dll"), b"fixture").unwrap();
        assert!(resolve_at(&executable).is_ok());

        let unexpected = ocr.join("unreviewed.txt");
        fs::write(&unexpected, b"fixture").unwrap();
        assert!(matches!(
            resolve_at(&executable),
            Err(AppError::OcrEngineUnavailable)
        ));
        assert_eq!(runtime_status_at(&executable), "unavailable");
        fs::remove_file(&unexpected).unwrap();

        // A nonempty DLL with an unreviewed basename must also be rejected.
        let unknown_dll = ocr.join("unreviewed.dll");
        fs::write(&unknown_dll, b"fixture").unwrap();
        assert!(matches!(
            resolve_at(&executable),
            Err(AppError::OcrEngineUnavailable)
        ));
        fs::remove_file(&unknown_dll).unwrap();
        assert!(resolve_at(&executable).is_ok());

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
    fn reviewed_dll_allowlist_matches_pinned_native_review_register() {
        // Keep the production runtime's fail-closed basename policy aligned
        // with the exact staged binaries whose hashes are reviewed in CI.
        let register =
            include_str!("../../../../docs/WINDOWS_OCR_NATIVE_BINARY_REVIEW_2026-10-12.md");
        let documented: std::collections::BTreeSet<String> = register
            .lines()
            .filter_map(|line| {
                let rest = line.strip_prefix("| `")?;
                let (name, _) = rest.split_once('`')?;
                name.to_ascii_lowercase()
                    .ends_with(".dll")
                    .then(|| name.to_ascii_lowercase())
            })
            .collect();
        let allowed: std::collections::BTreeSet<String> = REVIEWED_DLL_NAMES
            .iter()
            .map(|name| (*name).to_string())
            .collect();
        assert_eq!(documented.len(), 51, "review register must list 51 DLLs");
        assert_eq!(allowed, documented, "runtime DLL allowlist drifted");
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
            fs::write(ocr.join(format!("extra-{index:03}.dll")), b"fixture").unwrap();
        }
        assert!(matches!(
            resolve_at(&executable),
            Err(AppError::OcrEngineUnavailable)
        ));
    }
}
