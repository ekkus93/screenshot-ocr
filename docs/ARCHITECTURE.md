# Architecture

The React frontend presents state and submits typed requests. Rust owns capture, image validation, OCR, cleanup, clipboard policy, settings, diagnostics, app activation routing, and privacy-sensitive behavior.

## Rust boundaries

- `models.rs`: internal and serialized domain models.
- `error.rs`: typed internal failures and stable public errors.
- `state.rs`: single-capture ownership.
- `actions.rs`: second-instance, tray, startup, and shortcut action routing.
- `desktop.rs`: shared tray setup and registration of the shortcut selected by the platform service.
- `capture/`: the portable capture-backend contract only.
- `platform/`: the compile-time composition root and narrow `PlatformServices` contract.
- `platform/linux/`: Linux environment probing, GNOME Screenshot, XDG Portal, Linux shortcut policy, and Linux OCR-runtime discovery.
- `platform/windows/`: Windows capture, Windows shortcut policy, and packaged OCR-runtime discovery.
- `image_pipeline/`: bounded decode and deterministic in-memory variants.
- `ocr/`: platform-neutral Tesseract subprocess control, bounded language probing, cleanup, and candidate scoring. The platform layer supplies an internal `OcrRuntime` descriptor; the frontend never supplies executable paths.
- `settings.rs`: schema validation, atomic persistence, corrupt-settings quarantine, and safe recovery DTOs.
- `diagnostics.rs`: redacted capability report.
- `app.rs`: orchestration service.
- `commands.rs`: thin Tauri adapters.

No frontend payload can contain an executable path, filesystem path, shell fragment, or arbitrary OCR configuration.

### Portable-core dependency direction

Shared application policy depends only on portable domain types and the `PlatformServices` capability contract. Concrete Linux and Windows adapters depend inward on those contracts; portable modules do not import concrete platform implementations.

```text
React / TypeScript UI
        |
Tauri commands and shared desktop actions
        |
Portable Rust orchestration
        |
PlatformServices capability contract
      /   \
 Linux     Windows
 adapters  adapters
```

Platform adapters own mechanisms such as screen capture, desktop capability probing, target-specific shortcut selection, and OCR-runtime location. Shared code owns job reservation, cancellation, capture/OCR sequencing, image processing, candidate scoring, settings policy, clipboard recovery policy, public errors, and action routing.

The repository-policy job runs `scripts/check-platform-boundaries.py` to reject known concrete platform imports and OS-probing patterns outside `src-tauri/src/platform/`. This is a regression guard, not a substitute for architecture review.

## Frontend boundary

Production components must not import from `src/test`. Shared production controller types live under `src/app/`. Test-only helpers remain under `src/test/`.

Settings UI controls must represent implemented behavior truthfully. Reserved pre-release fields may remain in the schema for compatibility, but active-looking controls for notifications, autostart, close-to-tray, or other deferred features must either be disabled with explicit copy or removed from the UI.

## Activation and desktop integration

The action router accepts only explicit actions: `capture`, `cancel`, `toggle`, `show`, and `quit`. Unknown second-instance arguments show the main window instead of starting capture implicitly.

The tray menu currently exposes only implemented actions: capture, cancel, show, and quit. A Settings tray action is deferred until frontend route/focus support is implemented. Start-at-login and close-to-tray behavior are also deferred and must not be documented as active behavior.

On Linux X11, the app attempts direct `Super+Shift+O` shortcut registration and reports the result in diagnostics. On GNOME Wayland, users must create a custom shortcut that launches `screenshot-ocr capture`; the app does not modify GNOME settings automatically.

## Settings recovery contract

`get_settings` returns a settings-load result with settings plus an optional safe recovery warning. If settings are corrupt or invalid, the backend recovers with defaults, preserves/quarantines the corrupt file according to the settings policy, records a safe diagnostic code, and returns a warning that does not include raw JSON, filesystem paths, or user content.

## Clipboard failure contract

If OCR succeeds but immediate clipboard writing fails, the command returns an OCR result with recognized text, `copied = false`, and a retryable `clipboard_write_failed` warning. The frontend keeps the text editable and keeps manual copy retry available.

## OCR fixture policy

Pure cleanup and scoring tests may use synthetic text fixtures. They must not require Tesseract, screenshots, desktop APIs, clipboard state, or private real-world OCR output. See `docs/OCR_SYNTHETIC_FIXTURES.md`.

## Extension rules

A capture backend implements noninteractive capability probing and returns validated pixels with unambiguous cleanup ownership. An OCR engine accepts a prepared in-memory image and returns literal text without generative rewriting. New implementations require deterministic tests, typed failures, and privacy review.
