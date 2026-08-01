# ADR 0001: v0.1 product and platform decisions

- **Status:** accepted
- **Date:** 2026-08-01

## Decisions

- Product and binary name: `Screenshot OCR` / `screenshot-ocr`.
- Identifier: `io.github.ekkus93.screenshot-ocr`.
- Default branch remains `master` by owner direction.
- License: MIT.
- English is the only required v0.1 OCR language.
- Preview before copy is enabled by default.
- Documented shortcut: `Super+Shift+O`, configured as a GNOME custom shortcut.
- Persistent history is omitted in v0.1.
- `.deb` is mandatory; AppImage and Flatpak are deferred.
- Tesseract integration uses a bounded subprocess with fixed arguments and PNG data over stdin. This avoids a native binding ABI while keeping images off disk during OCR.
- The GNOME screenshot helper is the required Ubuntu compatibility backend. Portal area capture remains capability-gated and cannot be selected until its area target is proven.
