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
    #[cfg(target_os = "linux")]
    pub fn new(executable: PathBuf, tessdata_dir: Option<PathBuf>) -> Self {
        Self {
            executable,
            tessdata_dir,
            working_directory: None,
        }
    }
}
