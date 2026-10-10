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
    pub fn new(executable: PathBuf, tessdata_dir: Option<PathBuf>) -> Self {
        Self {
            executable,
            tessdata_dir,
            working_directory: None,
        }
    }

    /// Keep subprocess lookup rooted in an application-controlled directory.
    pub fn with_working_directory(mut self, directory: PathBuf) -> Self {
        self.working_directory = Some(directory);
        self
    }
}
