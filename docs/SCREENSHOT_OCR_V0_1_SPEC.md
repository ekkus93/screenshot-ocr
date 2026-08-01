# Screenshot OCR v0.1 Product and Technical Specification

**Repository:** `ekkus93/screenshot-ocr`  
**Document status:** Initial implementation baseline  
**Version:** 0.1  
**Date:** 2026-08-01  
**Initial platforms:** Ubuntu 22.04 LTS and Ubuntu 24.04 LTS, GNOME on Wayland or X11

## 1. Executive summary

Screenshot OCR is a privacy-first Linux desktop utility for extracting text from an arbitrary screen region and placing the recognized text in the clipboard. Its primary use case is copying text from terminal user interfaces such as Claude Code, where ordinary mouse selection and clipboard behavior may be inconvenient or unreliable.

The normal interaction is:

1. The user presses a configured shortcut or chooses **Capture text from screen** from the application or tray menu.
2. The desktop presents a region selector.
3. The user drags around the text to capture.
4. Screenshot OCR loads the selected image, removes the temporary image as soon as practical, preprocesses the pixels, and performs OCR locally.
5. The user either receives the recognized text directly in the clipboard or reviews and edits it before copying, depending on settings and confidence.
6. The application reports success or an actionable error without logging screenshot or recognized-text contents.

The application is implemented as a Tauri 2 desktop application. The frontend uses React, TypeScript, Vite, and Tailwind CSS. The privileged and platform-specific work is implemented in Rust. Tesseract is the initial OCR engine. Capture, OCR, preprocessing, clipboard, configuration, and desktop integration are separate Rust components with explicit interfaces so that later engines and Linux desktop backends can be added without rewriting the application.

## 2. Problem statement

Terminal applications often render selectable-looking text without providing a convenient, predictable copy workflow. The user may currently need to:

1. take a screenshot;
2. open or upload it elsewhere;
3. perform OCR in an unrelated application;
4. clean up the result manually; and
5. copy the result into the intended destination.

This workflow is slow and may disclose sensitive terminal contents to a third party. Developer-oriented terminal text also contains punctuation, whitespace, code, command-line flags, paths, hashes, and error messages that generic document OCR cleanup can damage.

Screenshot OCR reduces this operation to one shortcut and one drag gesture while keeping processing local.

## 3. Product goals

### 3.1 Primary goals

- Capture a user-selected region of the screen on supported Ubuntu systems.
- Recognize English terminal, source-code, and ordinary UI text locally.
- Preserve line breaks, indentation, blank lines, and developer punctuation as faithfully as possible.
- Copy recognized text into the desktop clipboard.
- Provide an optional preview/editor before copying.
- Support Ubuntu 22.04 and Ubuntu 24.04 on GNOME Wayland and GNOME X11.
- Operate from a keyboard shortcut, the main window, and a tray menu where tray support is available.
- Avoid retaining screenshots or recognized text by default.
- Produce clear, actionable failures rather than silent fallbacks.
- Keep the architecture extensible to additional OCR engines, languages, and desktop environments.

### 3.2 Secondary goals

- Start quickly and remain lightweight while idle.
- Detect relevant runtime capabilities and explain missing dependencies.
- Provide useful confidence and diagnostics without exposing captured content.
- Allow users to choose between preview-first and copy-immediately workflows.
- Package the application as a native Ubuntu `.deb` built from an Ubuntu 22.04 baseline.

## 4. Non-goals for v0.1

The following are explicitly outside the first release:

- Cloud OCR or upload-based processing.
- Translation.
- Continuous OCR, screen reading, or live transcription.
- General screen recording.
- Automatic interaction with Claude Code or any other terminal application.
- Accessibility screen-reader replacement.
- Table reconstruction beyond preserving recognized spacing and lines.
- Handwriting recognition.
- Mathematical formula recognition.
- PDF OCR.
- Windows or macOS support.
- Certified support for KDE Plasma, Sway, Hyprland, XFCE, MATE, or other desktop environments.
- Bundled PaddleOCR or large machine-learning runtimes.
- Automatic spelling correction of recognized terminal text.
- Persistent screenshot storage.
- Synchronization between computers.

## 5. Target users and primary user stories

### 5.1 Primary user

A Linux developer who works in terminal applications and needs to copy visible text that is awkward to select normally.

### 5.2 User stories

- As a user, I can press `Super+Shift+O`, select a region, and receive recognized text in my clipboard.
- As a user, I can start capture from the tray or main window when the shortcut is unavailable.
- As a user, I can review and edit recognized text before copying it.
- As a user, I can choose immediate copying when I prefer speed over review.
- As a developer, I can preserve indentation, shell commands, paths, flags, and source-code punctuation.
- As a privacy-conscious user, I can use the application with no network connection and no persistent history.
- As an Ubuntu 22.04 user, I can use the GNOME region-capture fallback even when the newest screenshot portal features are unavailable.
- As an Ubuntu 24.04 user, I can use the best available capture backend selected at runtime.
- As a user, I receive a clear explanation when Tesseract, English language data, a capture helper, or clipboard access is unavailable.
- As a user, I can cancel region selection without receiving an alarming error or losing previous preview text.

## 6. Product principles

1. **Local by default:** Captured pixels and recognized text do not leave the machine.
2. **No silent degradation:** A fallback must be observable in diagnostics and must preserve the security contract.
3. **Content minimization:** Images and OCR text live only as long as needed unless the user explicitly enables history in a later release.
4. **Terminal fidelity over prose correction:** Preserve literal characters and whitespace; do not apply generic autocorrection.
5. **Capability-driven Linux support:** Detect available portal and desktop facilities instead of relying only on distribution version strings.
6. **Thin frontend, authoritative Rust core:** The frontend presents state; Rust owns capture, OCR, cleanup, clipboard, settings validation, and privacy-sensitive behavior.
7. **Actionable errors:** Every user-facing failure should identify the failed stage and the next reasonable corrective action.

## 7. Supported platform matrix

| Platform | Session | Required v0.1 support | Preferred capture path | Required fallback |
|---|---|---:|---|---|
| Ubuntu 22.04 GNOME | Wayland | Yes | Screenshot portal when capable | `gnome-screenshot --area --file=...` |
| Ubuntu 22.04 GNOME | X11 | Yes | Portal or GNOME helper | `gnome-screenshot --area --file=...` |
| Ubuntu 24.04 GNOME | Wayland | Yes | Screenshot portal area target when advertised | GNOME helper |
| Ubuntu 24.04 GNOME | X11 | Yes | Portal or GNOME helper | GNOME helper |

### 7.1 Support interpretation

- “Supported” means capture, OCR, preview/copy, clipboard integration, settings, and packaged installation are validated on that platform.
- Other distributions or desktop environments may work but must be reported as unverified until covered by dedicated validation.
- Runtime behavior is chosen from detected capabilities. The application must not assume that every Ubuntu installation has identical portal backend versions or desktop packages.

## 8. Technology stack

### 8.1 Desktop shell and frontend

- Tauri 2
- React
- TypeScript with strict type checking
- Vite
- Tailwind CSS
- Accessible native HTML controls where possible

### 8.2 Rust backend

- Stable Rust toolchain pinned through `rust-toolchain.toml`
- Tokio only where asynchronous I/O or process management materially benefits the implementation
- Serde for command and configuration models
- `thiserror` or an equivalent typed-error approach
- `tracing` with mandatory redaction rules
- Tauri plugins or core APIs for clipboard, single-instance handling, notifications, tray integration, and autostart where appropriate

### 8.3 OCR and image processing

- Tesseract 5 as the first OCR engine
- English trained data as the initial required language pack
- Rust image decoding and preprocessing using a maintained image library
- Temporary PNG input only where required by the capture or OCR integration

### 8.4 Packaging

- Native Debian package (`.deb`) as the primary v0.1 artifact
- Ubuntu 22.04 build baseline for forward compatibility with Ubuntu 24.04
- AppImage and Flatpak deferred until the `.deb` release path is stable

## 9. High-level architecture

```text
┌───────────────────────────────────────────────────────────┐
│ React + TypeScript + Tailwind frontend                    │
│ Capture UI · Preview editor · Settings · Diagnostics      │
└───────────────────────┬───────────────────────────────────┘
                        │ typed Tauri commands and events
┌───────────────────────▼───────────────────────────────────┐
│ Tauri command/adaptor layer                               │
│ Validates requests · maps errors · emits state events     │
└───────────────────────┬───────────────────────────────────┘
                        │
┌───────────────────────▼───────────────────────────────────┐
│ Rust application service                                 │
│ Capture orchestration · cancellation · cleanup · policy   │
└───────┬───────────────┬───────────────┬───────────────────┘
        │               │               │
┌───────▼──────┐ ┌──────▼───────┐ ┌─────▼──────────┐
│ Capture      │ │ Preprocessing│ │ OCR engine     │
│ backends     │ │ pipeline     │ │ abstraction    │
└───────┬──────┘ └──────────────┘ └─────┬──────────┘
        │                                │
┌───────▼────────────────────────────────▼──────────┐
│ Clipboard · notifications · settings · telemetry │
│ privacy controls · desktop/session diagnostics   │
└──────────────────────────────────────────────────┘
```

## 10. Repository layout

The initial implementation should converge on the following organization:

```text
screenshot-ocr/
├── docs/
│   ├── SCREENSHOT_OCR_V0_1_SPEC.md
│   └── SCREENSHOT_OCR_V0_1_TODO.md
├── src/
│   ├── app/
│   ├── components/
│   ├── features/
│   │   ├── capture/
│   │   ├── preview/
│   │   ├── settings/
│   │   └── diagnostics/
│   ├── lib/
│   │   ├── tauri.ts
│   │   └── types.ts
│   ├── styles/
│   └── main.tsx
├── src-tauri/
│   ├── capabilities/
│   ├── icons/
│   ├── src/
│   │   ├── app.rs
│   │   ├── commands/
│   │   │   ├── capture.rs
│   │   │   ├── clipboard.rs
│   │   │   ├── diagnostics.rs
│   │   │   └── settings.rs
│   │   ├── capture/
│   │   │   ├── mod.rs
│   │   │   ├── portal.rs
│   │   │   ├── gnome_screenshot.rs
│   │   │   └── environment.rs
│   │   ├── ocr/
│   │   │   ├── mod.rs
│   │   │   ├── tesseract.rs
│   │   │   └── models.rs
│   │   ├── image_pipeline/
│   │   │   ├── mod.rs
│   │   │   ├── variants.rs
│   │   │   └── scoring.rs
│   │   ├── clipboard/
│   │   ├── config/
│   │   ├── desktop/
│   │   ├── privacy/
│   │   ├── error.rs
│   │   └── lib.rs
│   ├── tests/
│   ├── Cargo.toml
│   └── tauri.conf.json
├── tests/
│   ├── fixtures/
│   └── e2e/
├── package.json
├── tsconfig.json
├── vite.config.ts
├── tailwind.config.*
├── rust-toolchain.toml
└── README.md
```

Module names may change, but the boundaries must remain clear and independently testable.

## 11. Rust domain interfaces

### 11.1 Capture backend

```rust
#[async_trait::async_trait]
pub trait CaptureBackend: Send + Sync {
    fn id(&self) -> CaptureBackendId;
    async fn probe(&self) -> Result<CaptureCapability, CaptureError>;
    async fn capture_region(
        &self,
        request: CaptureRequest,
        cancellation: CancellationToken,
    ) -> Result<CapturedImage, CaptureError>;
}
```

Requirements:

- `probe` must not request capture permission or display an interactive dialog.
- The selected backend and fallback reason may be logged, but captured content and temporary file names must be redacted.
- Cancellation must be distinguished from failure.
- A backend must either return a validated image or an explicit typed error.
- A backend may not return a path whose ownership or cleanup responsibility is ambiguous.

### 11.2 OCR engine

```rust
pub trait OcrEngine: Send + Sync {
    fn id(&self) -> OcrEngineId;
    fn probe(&self) -> Result<OcrCapability, OcrError>;
    fn recognize(
        &self,
        image: &PreparedImage,
        options: &OcrOptions,
    ) -> Result<OcrCandidate, OcrError>;
}
```

### 11.3 Clipboard service

```rust
pub trait ClipboardService: Send + Sync {
    fn write_text(&self, text: &str) -> Result<(), ClipboardError>;
}
```

The clipboard implementation must verify that the write call succeeded. Where practical, integration tests should confirm that the value remains available after the capture operation completes.

### 11.4 Settings store

```rust
pub trait SettingsStore: Send + Sync {
    fn load(&self) -> Result<AppSettings, SettingsError>;
    fn save(&self, settings: &AppSettings) -> Result<(), SettingsError>;
}
```

Settings writes must be atomic and schema-validated. Corrupt settings must be quarantined or replaced only after a visible warning; they must not be silently discarded.

## 12. Core data models

```rust
pub struct CaptureRequest {
    pub mode: TextMode,
    pub language: LanguageSelection,
    pub copy_policy: CopyPolicy,
    pub source: CaptureSource,
}

pub struct CapturedImage {
    pub pixels: image::DynamicImage,
    pub width: u32,
    pub height: u32,
    pub backend: CaptureBackendId,
    pub cleanup_evidence: CleanupEvidence,
}

pub struct OcrCandidate {
    pub text: String,
    pub mean_confidence: Option<f32>,
    pub preprocessing_variant: PreprocessingVariant,
    pub warnings: Vec<OcrWarning>,
}

pub struct OcrResult {
    pub text: String,
    pub mean_confidence: Option<f32>,
    pub backend: CaptureBackendId,
    pub engine: OcrEngineId,
    pub preprocessing_variant: PreprocessingVariant,
    pub warnings: Vec<OcrWarning>,
    pub copied: bool,
    pub elapsed_ms: u64,
}
```

No serialized command response may include raw image bytes or a temporary image path.

## 13. Capture subsystem

### 13.1 Backend selection

At startup and when diagnostics are refreshed, the Rust core should determine:

- operating system and release;
- desktop environment;
- session type from `XDG_SESSION_TYPE` and related evidence;
- portal availability;
- screenshot portal interface version;
- advertised screenshot targets when available;
- presence and executable path of `gnome-screenshot`;
- Tesseract executable/library availability;
- installed language data;
- clipboard capability;
- tray capability where detectable.

Selection order:

1. Use the XDG Screenshot Portal with explicit area target only when the interface advertises that target.
2. Otherwise use the GNOME screenshot helper with area selection on supported Ubuntu GNOME environments.
3. If neither is available, return an unsupported-capture error with corrective guidance.

There must be no attempt to bypass Wayland isolation through unsafe or undocumented compositor access.

### 13.2 Portal backend

The portal backend must:

- query the interface version and available target bitmask;
- request the user-selected area only when advertised;
- handle asynchronous portal request completion;
- honor cancellation and user dismissal;
- validate the returned URI and image format;
- copy or decode the result into application-controlled memory;
- release any document-portal or temporary resource when feasible;
- avoid persisting the URI in logs.

### 13.3 GNOME screenshot backend

The Ubuntu compatibility backend launches the system helper using argument-safe process APIs, equivalent to:

```text
gnome-screenshot --area --file=<application-owned-temporary-path>
```

Requirements:

- Never invoke through a shell string.
- Create the temporary target with restrictive permissions in an application-owned runtime or temporary directory.
- Reject symlinks and unexpected file types.
- Validate that the process exited successfully and that a decodable image exists.
- Treat user cancellation separately from execution failure.
- Load the image and delete the file immediately afterward.
- Attempt cleanup on every exit path, including decode failure, OCR failure, clipboard failure, and application shutdown.
- Emit a redacted diagnostic event if cleanup fails.

### 13.4 Capture concurrency

Only one capture operation may own the region selector at a time.

If another capture request arrives while one is active:

- the default behavior is to focus or preserve the existing request and report `capture_already_active` to the second caller;
- the application must not launch multiple overlapping selectors;
- cancellation and state transitions must be race-safe.

### 13.5 Capture size limits

- Reject zero-size images.
- Enforce configurable maximum width, height, and decoded pixel count to prevent accidental memory exhaustion.
- Produce a specific `capture_too_large` error with instructions to select a smaller region.

## 14. Image preprocessing

### 14.1 Goals

- Improve recognition of antialiased terminal fonts.
- Handle light text on dark backgrounds and dark text on light backgrounds.
- Preserve code punctuation and whitespace.
- Avoid destructive transforms that invent or remove characters.

### 14.2 Initial variants

The pipeline should support a bounded set of deterministic candidates:

1. original image normalized to a supported color format;
2. grayscale with contrast normalization;
3. inverted grayscale when luminance analysis indicates a dark background;
4. 2× high-quality upscale plus contrast normalization;
5. 3× upscale for very small glyphs;
6. adaptive or Otsu-like thresholded variant where evidence supports it.

The number of variants attempted per capture must be bounded. The default fast path should avoid running every variant when the first result is already high-confidence and structurally plausible.

### 14.3 Candidate scoring

When multiple candidates are evaluated, scoring may consider:

- Tesseract mean confidence where available;
- fraction of replacement or control characters;
- ratio of alphanumeric and expected punctuation characters;
- suspiciously empty output relative to image size;
- implausibly long unbroken strings;
- line count and whitespace consistency;
- terminal-mode punctuation preservation.

Scoring must not apply language-model rewriting or external services.

## 15. OCR behavior

### 15.1 Initial engine

Tesseract is the required v0.1 engine. The application may integrate through a Rust binding or a carefully managed subprocess. The chosen approach must:

- avoid shell invocation;
- support cancellation or bounded execution time;
- capture exit status and structured diagnostics;
- prevent image or recognized text from appearing in logs;
- validate language data before capture begins where possible;
- provide deterministic configuration for tests.

### 15.2 Text modes

#### Terminal and source code mode — default

- Preserve line breaks and leading spaces.
- Preserve repeated interior spaces when Tesseract output provides them.
- Do not convert straight quotes to typographic quotes.
- Do not convert double hyphens to dashes.
- Do not run spelling correction.
- Normalize line endings to `\n` internally.
- Remove only clearly spurious trailing spaces unless a future fidelity option says otherwise.
- Preserve blank lines.

#### Normal document mode

- Prefer readable paragraph reconstruction.
- May collapse obviously accidental repeated spaces, but must not autocorrect words.

#### Single-line mode

- Configure Tesseract for a single text line.
- Return one normalized line while preserving literal punctuation.

Additional modes such as tables are deferred unless implementation is low-risk and covered by tests.

### 15.3 Confidence and review policy

Default policy:

- Show preview before copying.
- If the user enables immediate copy, copy automatically only when non-empty output is produced.
- Low confidence must produce a warning but should still show the text.
- Empty output must not overwrite the existing clipboard.
- OCR errors must never replace clipboard contents.

## 16. Clipboard behavior

- Clipboard writes occur only after successful OCR and policy evaluation.
- Empty or whitespace-only OCR output must not replace existing clipboard contents.
- In preview mode, clipboard contents are changed only when the user selects **Copy text**.
- In immediate-copy mode, the exact text returned to the frontend is the text written to the clipboard.
- A clipboard failure must leave the recognized text visible in preview with a retry action.
- The application must not read unrelated clipboard contents unless a narrowly scoped verification strategy requires it and the user has consented.

## 17. Frontend specification

### 17.1 Main navigation

The main window contains three logical sections:

1. **Capture**
2. **History** — shown as unavailable or disabled in v0.1 unless explicitly implemented
3. **Settings**

A diagnostics view may be a settings subsection or separate route.

### 17.2 Capture screen

Required elements:

- application state: Idle, Selecting region, Processing, Ready, Copied, Cancelled, or Error;
- primary **Capture text from screen** button;
- displayed shortcut and setup status;
- recognized-text editor using a monospace font;
- confidence indicator when available;
- warning list for suspicious or low-confidence output;
- **Copy text**, **Capture again**, and **Clear** actions;
- preview-before-copy toggle or a link to its setting;
- progress state that does not imply the app is frozen;
- accessible live-region announcements for state changes.

### 17.3 Settings screen

Required settings:

- OCR language, initially English;
- text mode, default Terminal and source code;
- preview before copying, default on;
- preserve indentation and blank lines, default on and mandatory for terminal mode unless intentionally overridden;
- notification after copy, default on;
- start at login, default off unless the user enables it;
- keep running in tray when window closes, default on when tray support is available;
- shortcut display and GNOME setup instructions;
- diagnostics and dependency status;
- reset settings action.

### 17.4 History screen

Persistent history is not required for v0.1. The screen should either be omitted or clearly state:

- history is off;
- captures and recognized text are not retained;
- no hidden history database exists.

Do not display fake history entries in the production application.

### 17.5 Window behavior

- Closing the main window hides it to the tray when tray operation is enabled and supported.
- Otherwise, closing exits cleanly after ensuring no capture or temporary cleanup remains pending.
- Starting a capture should hide the main window before the selector appears to prevent capturing the application itself.
- After cancellation, the prior window visibility should be restored according to the originating action and settings.
- After a successful capture, the main window appears for preview mode and may remain hidden for immediate-copy mode.

### 17.6 Accessibility

- Full keyboard navigation.
- Visible focus indicators.
- Properly associated labels and controls.
- State changes announced through ARIA live regions.
- No color-only error or confidence communication.
- UI must remain usable at 200% scaling and a narrow window width.

## 18. Shortcut and single-instance behavior

### 18.1 Default shortcut

The documented default is `Super+Shift+O`.

### 18.2 Wayland-compatible invocation

Because desktop-wide shortcut handling differs under Wayland, v0.1 must provide a command that GNOME can invoke, conceptually:

```text
screenshot-ocr capture
```

A GNOME custom keyboard shortcut may call this command. The application settings screen must show the exact installed binary command and setup guidance.

### 18.3 Single instance

- Only one primary application instance may own mutable application state.
- A second process started with `capture` must signal the existing instance to begin capture and then exit.
- If no primary instance exists, `screenshot-ocr capture` starts the application, initializes services, begins capture, and follows the configured preview/copy policy.
- Concurrent capture requests are serialized as described in section 13.4.

Direct global-shortcut registration may be enabled on environments where it is proven reliable, but it cannot be the only supported mechanism.

## 19. System tray

Required tray actions when available:

- Capture text
- Show Screenshot OCR
- Settings
- Quit

Requirements:

- Tray absence must not prevent the main application from functioning.
- The UI must explain when tray integration is unavailable.
- **Quit** must cancel or finish cleanup safely before process exit.
- Tray actions and frontend actions must call the same Rust application service rather than duplicate workflows.

## 20. Tauri command contract

Illustrative command surface:

```rust
#[tauri::command]
async fn start_capture(request: CaptureUiRequest) -> Result<CaptureJob, PublicError>;

#[tauri::command]
async fn cancel_capture(job_id: String) -> Result<(), PublicError>;

#[tauri::command]
async fn copy_text(text: String) -> Result<(), PublicError>;

#[tauri::command]
async fn get_settings() -> Result<AppSettingsDto, PublicError>;

#[tauri::command]
async fn update_settings(settings: AppSettingsDto) -> Result<AppSettingsDto, PublicError>;

#[tauri::command]
async fn get_diagnostics() -> Result<DiagnosticsDto, PublicError>;
```

Long-running capture and OCR state should be reported through typed Tauri events or a job-state query. Every event must include a job identifier so stale events cannot overwrite newer frontend state.

The frontend must not be allowed to pass executable paths, arbitrary command-line arguments, temporary paths, or raw Tesseract configuration strings to Rust.

## 21. Application state machine

```text
Idle
  └─ start → Preparing
Preparing
  ├─ ready → SelectingRegion
  ├─ dependency failure → Failed
  └─ cancel → Cancelled
SelectingRegion
  ├─ image selected → LoadingImage
  ├─ user dismisses → Cancelled
  └─ backend failure → Failed
LoadingImage
  ├─ valid image → Preprocessing
  └─ invalid image/cleanup issue → Failed
Preprocessing
  ├─ candidate ready → Recognizing
  └─ failure → Failed
Recognizing
  ├─ result → Reviewing or Copying
  ├─ empty result → ReviewWarning
  └─ failure → Failed
Reviewing
  ├─ user copies → Copied
  ├─ capture again → Preparing
  └─ clear → Idle
Copying
  ├─ success → Copied
  └─ failure → ReviewWithClipboardError
Copied
  └─ next action → Idle or Preparing
Cancelled
  └─ acknowledge/new request → Idle or Preparing
Failed
  └─ retry/new request → Idle or Preparing
```

State transitions must be enforced in Rust. The frontend mirrors authoritative state and must ignore stale job events.

## 22. Settings schema

Illustrative schema:

```json
{
  "schemaVersion": 1,
  "language": "eng",
  "textMode": "terminal",
  "previewBeforeCopy": true,
  "preserveWhitespace": true,
  "notifyAfterCopy": true,
  "startAtLogin": false,
  "closeToTray": true,
  "preferredCaptureBackend": "auto",
  "maxDecodedPixels": 50000000
}
```

Rules:

- Unknown fields are ignored only when forward compatibility is safe.
- Unknown enum values produce a migration or validation warning.
- Security and resource limits have hard application bounds that settings cannot exceed.
- The settings file contains no screenshot data, OCR text, clipboard contents, or terminal metadata.

## 23. Error model

Errors are typed internally and mapped to a stable public code plus safe human-readable message.

Required public error codes include:

- `capture_already_active`
- `capture_cancelled`
- `capture_backend_unavailable`
- `capture_permission_denied`
- `capture_process_failed`
- `capture_result_missing`
- `capture_image_invalid`
- `capture_too_large`
- `temporary_cleanup_failed`
- `ocr_engine_unavailable`
- `ocr_language_missing`
- `ocr_timed_out`
- `ocr_failed`
- `ocr_empty_result`
- `clipboard_unavailable`
- `clipboard_write_failed`
- `settings_invalid`
- `settings_write_failed`
- `unsupported_environment`
- `internal_error`

Public messages must not contain captured text, image paths, raw environment dumps, or subprocess output that might expose content.

## 24. Privacy and security requirements

### 24.1 Data handling

- No network request is required for normal operation.
- No analytics or crash-report upload is enabled by default.
- Captured images are processed in memory after capture whenever possible.
- Temporary image files are deleted immediately after decode and on all known failure paths.
- Recognized text is held in memory only for the current preview and clipboard operation.
- Persistent history is absent or disabled by default.
- Logs never contain screenshot pixels, OCR text, clipboard text, full temporary paths, or command content.

### 24.2 Process execution

- No shell interpolation.
- Executables are resolved from trusted configuration or a validated system path.
- Arguments are passed as an argument vector.
- Environment inheritance is minimized where practical.
- Subprocesses receive bounded execution time and cancellation handling.
- Exit status and safe stderr classifications are recorded without retaining content-bearing output.

### 24.3 Temporary files

- Restrictive permissions.
- Random, non-predictable names.
- Application-owned directory.
- No following symlinks.
- File type and size validation.
- Cleanup guard that runs during unwinding and normal returns.
- Startup scavenger removes only stale files that can be cryptographically or structurally identified as owned by this application.

### 24.4 Tauri security

- Minimal command allowlist/capabilities.
- No unrestricted shell plugin permissions.
- Content Security Policy appropriate for a local bundled frontend.
- No remote frontend content.
- No arbitrary URL navigation.
- No frontend access to filesystem paths except safe display-only diagnostics explicitly returned by Rust.

## 25. Diagnostics

The diagnostics view may expose:

- application version;
- Ubuntu release;
- desktop environment and session type;
- selected capture backend;
- portal availability, interface version, and area-target availability;
- GNOME screenshot helper availability and version;
- Tesseract availability and version;
- installed OCR languages;
- clipboard and tray integration status;
- settings schema version;
- last operation stage, duration, and safe error code;
- temporary cleanup health as counts, not paths.

Diagnostics must never expose captured text or image contents. A **Copy diagnostics** action must produce a redacted report suitable for an issue.

## 26. Notifications

When enabled:

- Successful immediate copy: **Text copied to clipboard**.
- Successful preview operation: no system notification is required because the window is visible.
- Cancellation: no notification by default.
- Background failure: concise notification with an action to open the app.

Notifications must not include recognized text.

## 27. Dependency strategy

### 27.1 Initial release decision

For v0.1, use system Tesseract and English language data as Debian package dependencies where practical. The package should verify them at startup and provide manual installation guidance for development or unsupported installation methods.

Likely runtime dependencies include:

- `tesseract-ocr`
- `tesseract-ocr-eng`
- `gnome-screenshot` for the compatibility capture backend
- Tauri-generated WebKitGTK and GTK runtime dependencies
- AppIndicator runtime support when tray integration requires it

Exact package metadata must be validated on both Ubuntu 22.04 and 24.04 rather than copied blindly into release configuration.

### 27.2 Future bundling

Bundling Tesseract and trained data may be evaluated later for portability, but only after reviewing:

- licensing and attribution;
- security update responsibility;
- binary size;
- dynamic library compatibility;
- language-pack distribution;
- reproducible builds.

## 28. Performance requirements

On a representative modern desktop for a typical terminal selection:

- Idle CPU usage should be effectively negligible.
- Idle memory should remain reasonable for a Tauri application.
- UI response to capture initiation should be perceptually immediate.
- The selector should appear without an unnecessary fixed delay.
- Median OCR-to-result target: under 2 seconds for an ordinary terminal region after selection.
- The application should remain responsive during OCR.
- A bounded timeout must stop pathological OCR operations.

Performance measurements must include image dimensions, selected preprocessing variant, and stage durations, but never captured content.

## 29. Testing strategy

### 29.1 Rust unit tests

- capture backend selection from capability matrices;
- environment/session detection;
- command argument construction;
- temporary-file ownership and cleanup guards;
- image size validation;
- luminance and inversion decisions;
- preprocessing variant generation;
- candidate scoring;
- text-mode cleanup rules;
- settings validation and atomic persistence;
- error redaction;
- state machine transition validity;
- stale job-event rejection logic where implemented in Rust.

### 29.2 Frontend unit/component tests

- capture state rendering;
- disabled and busy actions;
- preview editing and copy request;
- low-confidence and empty-result warnings;
- settings validation;
- diagnostics rendering;
- keyboard navigation and accessible labels;
- stale event handling by job identifier.

### 29.3 Fixture-based OCR tests

Create version-controlled, synthetic fixtures that do not contain private data:

- light terminal text on dark background;
- dark terminal text on light background;
- small monospace text;
- Rust compiler errors;
- shell commands with flags and paths;
- JSON, TOML, YAML, and Markdown punctuation;
- Unicode punctuation and selected non-ASCII text where supported;
- noisy antialiasing and fractional-scaling samples.

Tests should compare exact output where deterministic and structural properties where Tesseract versions legitimately vary.

### 29.4 Integration tests

- fake capture backend → preprocessing → fake OCR → preview;
- fake capture backend → OCR → clipboard test double;
- cancellation at every stage;
- OCR failure with guaranteed image cleanup;
- clipboard failure without losing preview text;
- corrupt settings recovery with visible warning;
- second-instance capture request routing;
- cleanup scavenger limited to owned stale artifacts.

### 29.5 Manual platform validation

Required test matrix:

- Ubuntu 22.04 GNOME Wayland
- Ubuntu 22.04 GNOME X11
- Ubuntu 24.04 GNOME Wayland
- Ubuntu 24.04 GNOME X11

For each environment validate:

- main-window capture;
- tray capture where supported;
- GNOME custom shortcut invocation;
- cancellation;
- repeat capture;
- clipboard paste into terminal, browser, and text editor;
- one and multiple monitors;
- 100%, 125%, 150%, and 200% scaling where available;
- dark and light terminal themes;
- logout/restart startup behavior;
- package install, upgrade, and removal;
- no residual temporary images after success, cancellation, forced helper failure, and application restart.

## 30. Continuous integration

Minimum CI gates:

### Frontend

- dependency lockfile integrity;
- TypeScript strict typecheck;
- lint;
- formatting check;
- unit/component tests;
- production build.

### Rust

- `cargo fmt --check`;
- `cargo clippy --all-targets --all-features -- -D warnings`;
- unit and integration tests;
- dependency/license/security checks using explicitly selected tools;
- release build on Ubuntu 22.04.

### Packaging

- produce `.deb` artifact;
- inspect declared dependencies;
- install in a clean Ubuntu 22.04 environment;
- launch smoke test under a virtual display where possible;
- preserve manual Wayland capture validation as a required release evidence item when CI cannot exercise it faithfully.

## 31. Release and acceptance criteria

v0.1 is complete only when all of the following are true:

1. A clean Ubuntu 22.04 and Ubuntu 24.04 installation can install the `.deb` with documented dependencies.
2. Region capture works on GNOME Wayland and X11 in both releases.
3. The application supports main-window, tray, and GNOME-shortcut capture paths, with a documented exception only where tray infrastructure is absent.
4. English terminal text can be recognized locally and copied.
5. Preview mode permits editing before clipboard mutation.
6. Immediate-copy mode never overwrites the clipboard on cancellation, empty OCR, or OCR failure.
7. Indentation, line breaks, and representative developer punctuation pass fixture and manual tests.
8. Temporary image cleanup is proven on success and all tested failure paths.
9. Logs and copied diagnostics contain no captured image, OCR text, clipboard text, or unredacted temporary path.
10. Missing dependencies and unsupported environments produce actionable errors.
11. Concurrent capture requests cannot create overlapping selectors or corrupt state.
12. CI gates pass on the exact release commit.
13. The release contains installation, shortcut setup, privacy, troubleshooting, and uninstall documentation.
14. Manual validation evidence exists for all four required Ubuntu/session combinations.
15. No known P0 or P1 defects remain open.

## 32. Severity definitions

- **P0:** Data exposure, arbitrary command execution, destructive behavior, persistent screenshot leakage, or application unusable on all supported systems.
- **P1:** Core capture/OCR/clipboard path broken on a required platform; clipboard overwritten incorrectly; cleanup contract violated; silent security fallback.
- **P2:** Important workflow defect with a reasonable workaround; material OCR fidelity regression; tray or shortcut problem with main-window fallback.
- **P3:** Cosmetic issue, minor accessibility defect, documentation gap, or low-impact enhancement.

## 33. Open decisions that do not block implementation

- Final license for Screenshot OCR.
- Whether the production name remains **Screenshot OCR**.
- Whether history is omitted entirely or represented by an explanatory disabled screen in v0.1.
- Whether direct Tesseract library integration or subprocess integration has the best reliability and packaging profile.
- Whether AppImage joins the first stable release after `.deb` validation.
- Which additional languages should be first after English.

Each decision must be documented before release, but none prevents scaffolding or core implementation.

## 34. Future roadmap

Potential post-v0.1 work:

- KDE/Spectacle backend.
- wlroots backend using user-approved compositor tools.
- AppImage and Flatpak packages.
- Additional Tesseract languages and language-pack management.
- Optional local-only encrypted history.
- PaddleOCR or another OCR engine behind the existing interface.
- Code-aware suspicious-character review without generative rewriting.
- Copy as Markdown code block.
- Prompt-prefix removal for common shells.
- Multiple OCR regions in one capture session.
- Optional local translation engine.
- Windows and macOS backends.

## 35. Authoritative implementation rule

When implementation behavior conflicts with this specification, the implementation is not implicitly authoritative. The discrepancy must be resolved by either:

1. changing the implementation to conform; or
2. updating this specification with rationale, tests, migration impact, and acceptance criteria.

Security, privacy, cleanup, and no-silent-fallback requirements may not be weakened merely to make a test pass.

## 36. Technical references

- Tauri 2 prerequisites and Linux WebKitGTK guidance: `https://v2.tauri.app/start/prerequisites/`
- Tauri Linux/AppImage baseline guidance: `https://v2.tauri.app/distribute/appimage/`
- XDG Screenshot Portal interface: `https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.Screenshot.html`
- Ubuntu 22.04 `gnome-screenshot` manual: `https://manpages.ubuntu.com/manpages/jammy/man1/gnome-screenshot.1.html`
- Tesseract OCR repository and documentation: `https://github.com/tesseract-ocr/tesseract`
