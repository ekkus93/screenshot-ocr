# Architecture

The React frontend presents state and submits typed requests. Rust owns capture, image validation, OCR, cleanup, clipboard policy, settings, diagnostics, and error redaction.

## Rust boundaries

- `models.rs`: internal and serialized domain models.
- `error.rs`: typed internal failures and stable public errors.
- `state.rs`: single-capture ownership.
- `capture/`: environment probing and GNOME helper backend.
- `image_pipeline/`: bounded decode and deterministic in-memory variants.
- `ocr/`: Tesseract subprocess, cleanup, and candidate scoring.
- `settings.rs`: schema validation and atomic persistence.
- `diagnostics.rs`: redacted capability report.
- `app.rs`: orchestration service.
- `commands.rs`: thin Tauri adapters.

No frontend payload can contain an executable path, filesystem path, shell fragment, or arbitrary OCR configuration.

## Extension rules

A capture backend implements noninteractive capability probing and returns validated pixels with unambiguous cleanup ownership. An OCR engine accepts a prepared in-memory image and returns literal text without generative rewriting. New implementations require deterministic tests, typed failures, and privacy review.
