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
    // The parent test uses a different working directory. Never record or log
    // the path: fail before the readiness marker if child CWD is not pinned.
    if let Some(expected) = env::var_os("SCREENSHOT_OCR_WINDOWS_FAKE_EXPECTED_CWD") {
        if !env::current_dir().is_ok_and(|cwd| cwd == PathBuf::from(expected)) {
            std::process::exit(2);
        }
    }
    // The shared engine must not pass a developer-controlled DLL/executable
    // search path into the app-owned OCR child process.
    if env::var_os("SCREENSHOT_OCR_WINDOWS_FAKE_EXPECT_NO_PATH").is_some()
        && env::var_os("PATH").is_some()
    {
        std::process::exit(3);
    }
    // The CI harness injects synthetic hostile native search overrides into
    // the parent process. The packaged OCR command must strip them before
    // spawning this child, without depending on actual developer settings.
    if env::var_os("SCREENSHOT_OCR_WINDOWS_FAKE_EXPECT_NO_NATIVE_CONFIG").is_some() {
        for key in [
            "GIO_EXTRA_MODULES",
            "FONTCONFIG_FILE",
            "OPENSSL_CONF",
            "QT_PLUGIN_PATH",
        ] {
            if env::var_os(key).is_some() {
                std::process::exit(4);
            }
        }
    }
    fs::write(directory.join(format!("{mode}.pid")), std::process::id().to_string())
        .expect("write synthetic process id");
    fs::write(directory.join(format!("{mode}.started")), b"ready")
        .expect("write synthetic start marker");
    thread::sleep(Duration::from_secs(60));
}
