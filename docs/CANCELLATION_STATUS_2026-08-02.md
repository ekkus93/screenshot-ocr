# Capture Cancellation Status — 2026-08-02

**Repository:** `ekkus93/screenshot-ocr`  
**Branch:** `master`  
**Validated source commit:** `156dd467190ab74b535eff610ff42558f633dd53`  
**Status:** Hosted-CI validated; physical GNOME/Wayland validation and reachable hidden-window cancellation remain open

## Implemented behavior

The capture pipeline now uses a caller-owned `CaptureJobId` and an application-level cancellation token shared across the Rust state machine, capture backend, and OCR engine.

The implementation:

- creates the job identifier in the frontend before invoking `start_capture`;
- associates the pending Rust command, cancellation request, and final result with the same job identifier;
- rejects a second capture while another job still owns the selector;
- signals cancellation without releasing job ownership early;
- releases ownership only after the pending capture command has completed its cleanup path;
- rejects stale cancellation job identifiers;
- distinguishes user cancellation from backend, OCR, clipboard, and internal failures;
- prevents a cancelled job from writing immediate-copy text to the clipboard;
- preserves the previous recognized text until a replacement capture succeeds;
- restores the hidden Tauri window after success, cancellation, capture failure, OCR failure, or immediate-copy clipboard failure.

## Backend and subprocess lifecycle

### GNOME screenshot helper

The GNOME backend:

- spawns `gnome-screenshot` directly with an argument vector;
- waits on the helper with bounded lifetime;
- selects between normal completion, cancellation, and timeout;
- explicitly kills and reaps the helper after cancellation or timeout;
- preserves temporary-directory cleanup on all return paths;
- maps helper exit code `1` to user cancellation and other unsuccessful exits to capture failure.

### Tesseract

The Tesseract backend:

- owns the spawned child process;
- streams bounded PNG input and bounded OCR output;
- selects between process completion, cancellation, and timeout;
- explicitly kills and reaps Tesseract after cancellation or timeout;
- propagates cancellation instead of treating it as a failed OCR candidate;
- does not expose or log OCR text during failure handling.

### XDG Screenshot Portal

The portal backend:

- checks cancellation before and during capability probing;
- checks cancellation before and during the screenshot request;
- returns the stable `capture_cancelled` application error when cancellation wins;
- does not open the GNOME selector after a portal selector has already been requested.

The current `ashpd` request call is cancelled by dropping the pending future. Explicit portal request-object closure is not yet claimed and still requires implementation or physical evidence.

## Frontend behavior

The React controller:

- generates a UUID job identifier before each capture;
- rejects stale completion events with a frontend request token;
- exposes a `cancel()` action using the active job identifier;
- enters a distinct `cancelling` state;
- preserves prior editor text when capture is cancelled;
- clears the active job reference only when the original `start_capture` invocation resolves.

The capture panel includes an accessible **Cancel capture** control and disables unrelated capture/editor actions while the job is busy.

## Important UI reachability limitation

The Tauri main window is hidden before the native selector is opened and remains hidden while OCR executes. Therefore, the React **Cancel capture** button is not directly clickable during the normal hidden-window capture flow.

The implemented cancellation path is still real and is exercised by native selector dismissal, backend cancellation signaling, and automated Rust tests. However, application-triggered cancellation is not yet fully reachable during the hidden interval. A later batch must provide one of these mechanisms:

- a global cancellation shortcut;
- a tray-menu cancellation action;
- or a visible progress/cancellation window after region selection.

Until that work and physical validation are complete, this document does not claim that the visible Cancel button alone provides end-to-end desktop cancellation.

## Automated coverage

The hosted test suite covers:

- idempotent cancellation tokens;
- cancellation notification without a lost-wakeup race;
- caller-owned job identifiers;
- prevention of overlapping capture ownership;
- cancellation without premature ownership release;
- start/cancel/start ownership behavior;
- stale job identifier rejection;
- cancellation before portal access;
- cancellation propagation through orchestration;
- immediate-copy cancellation checks;
- clipboard input validation;
- frontend stale-result rejection and status typing through the mandatory frontend suite.

Physical process and desktop integration remain separate validation boundaries.

## Hosted validation evidence

The permanent read-only workflow passed all mandatory jobs on the exact same commit:

```text
COMMIT_SHA=156dd467190ab74b535eff610ff42558f633dd53
CI_RUN_ID=30730812349
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
PACKAGE_SIZE_BYTES=5826032
INSTALLED_SIZE_KIB=16898
PACKAGE_SHA256=f37e87d2621b4b9a63eee59ea38fe22780d0dcfbeb582a604a891aa345c92b56
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
ARTIFACT_ID=8827984069
ARTIFACT_NAME=screenshot-ocr-deb-156dd467190ab74b535eff610ff42558f633dd53
ARTIFACT_ARCHIVE_SIZE_BYTES=5794892
ARTIFACT_ARCHIVE_SHA256=0b9dc3398fbac0a9bd405ffdf3c03e4d2f9fcd7bcfaf33c53a4f71c041901a59
ARTIFACT_EXPIRES=2026-10-31T03:32:54Z
```

## Remaining validation and implementation

Hosted CI proves type safety, formatting, linting, unit behavior, compilation, and package construction. It does not prove the real desktop interaction.

Still required on Ubuntu 24.04 GNOME Wayland:

- cancel the GNOME selector with Escape and confirm no helper remains;
- cancel during OCR through a reachable application control and confirm no Tesseract process remains;
- verify the window is restored after cancellation and every failure path;
- verify no clipboard write occurs after cancellation;
- verify start/cancel/start cannot display overlapping selectors;
- validate portal cancellation and permission denial;
- determine whether explicit portal request closure is required and supported;
- verify tray or global-shortcut cancellation while the main window is hidden;
- install and exercise the validated Debian package.

The authoritative TODO should only mark items complete when their required automated or physical evidence exists.