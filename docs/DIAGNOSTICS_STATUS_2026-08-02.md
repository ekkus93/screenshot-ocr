# Runtime Diagnostics Status — 2026-08-02

**Repository:** `ekkus93/screenshot-ocr`  
**Branch:** `master`  
**Validated source commit:** `2fdce81e5163c5234515fc26efeea48544077c88`  
**Status:** Hosted-CI validated; physical desktop verification remains open

## Implemented behavior

The application now owns a privacy-safe, in-memory runtime diagnostics recorder through `AppServices`.

The recorder retains only:

- the most recent stable public error code;
- a monotonic count of temporary-cleanup failures.

It does not retain:

- captured pixels;
- recognized text;
- clipboard text;
- temporary file paths;
- executable paths;
- portal result URIs;
- raw subprocess output;
- raw D-Bus errors.

The diagnostics report includes:

- application version;
- OS release;
- desktop environment;
- session type;
- safe XDG Screenshot Portal capability summary;
- GNOME screenshot helper availability;
- Tesseract availability;
- installed OCR language codes;
- clipboard integration status;
- tray integration status;
- settings schema version;
- most recent stable public error code;
- cleanup-failure count.

## Portal capability reporting

The portal diagnostics probe is bounded and content-free.

It reports one of a small set of safe summaries, including:

```text
Screenshot v3; area target available
Screenshot v3; area target unavailable
Screenshot portal unavailable or probe timed out
Screenshot portal capability probe failed
session bus unavailable
```

The report does not expose D-Bus addresses, object paths, portal result URIs, screenshot paths, or backend error strings.

The same portal capability model is used by capture selection and diagnostics, avoiding the previous stale static summary.

## Error and cleanup recording

Expected public failures are converted to the existing stable `ErrorCode` enum before they are recorded.

The recorder is updated for failures returned by:

- capture orchestration;
- immediate clipboard copy;
- explicit clipboard copy;
- cancellation requests;
- settings load, save, and reset;
- environment diagnostics;
- window restoration.

A corrupt settings file still falls back to default settings, but the stable `settings_invalid` code is now retained for diagnostics. A separate user-visible corrupt-settings warning remains open.

Only `temporary_cleanup_failed` increments the cleanup-failure counter. A later error may replace the displayed last error code without resetting or decrementing the cleanup count.

## Automated coverage

The hosted Rust suite verifies:

- executable paths are absent from serialized diagnostics;
- runtime diagnostics retain only stable codes and numeric counts;
- cleanup failures increment independently of the last-error field;
- public-error recording updates the shared application diagnostics state;
- portal capability classification preserves version and area-target state;
- a pre-cancelled portal probe exits without portal access;
- all pre-existing capture, cancellation, OCR, settings, and image-bound tests remain green.

The frontend suite verifies that the diagnostics DTO remains type-compatible and renderable through the existing diagnostics panel.

## Hosted validation evidence

The permanent read-only workflow passed all mandatory jobs on the exact same source commit:

```text
COMMIT_SHA=2fdce81e5163c5234515fc26efeea48544077c88
CI_RUN_ID=30731505719
REPOSITORY_POLICY=success
FRONTEND_QUALITY=success
RUST_QUALITY=success
DEBIAN_PACKAGE_SMOKE=success
```

The frontend job passed:

- Prettier formatting;
- ESLint with zero accepted warnings;
- TypeScript checking;
- frontend tests;
- production Vite build.

The Rust job passed:

- Rustfmt;
- Clippy across all targets and features with `-D warnings`;
- Rust tests;
- Rust workspace build.

## Debian package evidence

The Ubuntu 22.04 package job built and inspected:

```text
PACKAGE_FILE=Screenshot OCR_0.1.0_amd64.deb
PACKAGE_NAME=screenshot-ocr
PACKAGE_VERSION=0.1.0
ARCHITECTURE=amd64
PACKAGE_SIZE_BYTES=5826646
INSTALLED_SIZE_KIB=16898
PACKAGE_SHA256=b079ee4bd5a2ea64c91b51248105c15cf6cc3da14a4b2f643f0f664f29ecf180
```

Declared package dependencies:

```text
gnome-screenshot
tesseract-ocr
tesseract-ocr-eng
libwebkit2gtk-4.1-0
libgtk-3-0
```

Artifact evidence:

```text
ARTIFACT_ID=8828666725
ARTIFACT_NAME=screenshot-ocr-deb-2fdce81e5163c5234515fc26efeea48544077c88
ARTIFACT_ARCHIVE_SIZE_BYTES=5794424
ARTIFACT_ARCHIVE_SHA256=971cf67e5b70bf0e47fc7a54454507e74955d3e6ad0bb5c0291c488174680351
ARTIFACT_EXPIRES=2026-10-31T03:58:59Z
```

## Evidence boundary

Hosted CI proves type safety, formatting, linting, unit behavior, compilation, and package construction. It does not prove the values reported by a real GNOME/Wayland desktop session.

Still required on Ubuntu 24.04 GNOME Wayland:

- verify the reported portal version and area-target state against the installed portal stack;
- verify helper and Tesseract availability reporting after package installation;
- cause safe representative failures and verify the displayed stable codes;
- cause or simulate a real cleanup failure without exposing a sensitive path;
- verify diagnostics remain responsive when the session bus is absent or unavailable;
- verify no screenshot, OCR, clipboard, executable-path, or temporary-path content appears in the UI or logs.

## Remaining diagnostics implementation

The following M3.3 items remain open:

- GNOME screenshot helper version, not merely presence;
- Tesseract version, not merely presence;
- last completed stage duration;
- **Copy diagnostics** with guaranteed redaction;
- a complete deterministic snapshot test for the rendered diagnostics report;
- physical validation of all reported fields.

The authoritative TODO should mark only individually proven diagnostics items complete; it must not mark the M3 acceptance gate complete yet.
