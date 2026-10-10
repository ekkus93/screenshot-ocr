mod cleanup;
mod tesseract;

use std::path::PathBuf;

pub use cleanup::select_best_candidate;
pub use tesseract::TesseractEngine;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OcrRuntime {
    pub executable: PathBuf,
    pub tessdata_dir: Option<PathBuf>,
    pub working_directory: Option<PathBuf>,
}

impl OcrRuntime {
    // Windows uses the packaged descriptor directly; tests still exercise this
    // portable constructor until the native capture adapter is available.
    #[cfg_attr(target_os = "windows", allow(dead_code))]
    pub fn new(executable: PathBuf, tessdata_dir: Option<PathBuf>) -> Self {
        Self {
            executable,
            tessdata_dir,
            working_directory: None,
        }
    }
}
