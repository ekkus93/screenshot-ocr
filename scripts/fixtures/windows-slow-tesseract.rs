// Synthetic Windows OCR child for cancellation tests; no user data or shell.
use std::{env, fs, path::PathBuf, thread, time::Duration};

fn main() {
    let mode = if env::args().nth(1).as_deref() == Some("--list-langs") {
        "probe"
    } else {
        "recognize"
    };
    let directory = PathBuf::from(
        env::var_os("SCREENSHOT_OCR_WINDOWS_FAKE_MARKER_DIR")
            .expect("synthetic marker directory required"),
    );
    fs::write(directory.join(format!("{mode}.pid")), std::process::id().to_string())
        .expect("write synthetic process id");
    fs::write(directory.join(format!("{mode}.started")), b"ready")
        .expect("write synthetic start marker");
    thread::sleep(Duration::from_secs(60));
}
