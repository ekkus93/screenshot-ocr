mod cleanup;
mod tesseract;

use std::path::PathBuf;

pub use cleanup::select_best_candidate;
pub use tesseract::TesseractEngine;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OcrRuntime {
    pub executable: PathBuf,
    pub tessdata_dir: Option<PathBuf>,
}

impl OcrRuntime {
    pub fn new(executable: PathBuf, tessdata_dir: Option<PathBuf>) -> Self {
        Self {
            executable,
            tessdata_dir,
        }
    }
}
