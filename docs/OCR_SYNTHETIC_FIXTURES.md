# Synthetic OCR fixtures

Screenshot OCR tests must never commit private screenshots, real terminal captures, or real OCR output copied from a user's screen.

This document defines the safe fixture policy for v0.1 testing.

## Allowed fixtures

Allowed fixtures are synthetic text fixtures that are typed by a developer or generated deterministically from hard-coded fake examples. They may include developer punctuation that the cleanup logic must preserve, such as:

```text
cargo run --bin screenshot-ocr -- --mode=terminal
assert_eq!(foo::<Bar>(), value[0]);
error: expected `;`
```

Fixtures may use obviously fake markers such as `SYNTHETIC_OCR_FIXTURE_9f33` so leakage/redaction tests can detect accidental propagation without exposing real secrets.

## Disallowed fixtures

Do not commit:

- screenshots from a real desktop session;
- OCR output copied from a real terminal, editor, browser, email, chat, or document;
- clipboard text from a real user workflow;
- package, log, or CI artifacts containing screenshots or recognized text;
- executable paths, temporary paths, portal result URIs, raw helper stderr, or raw helper stdout from a real machine.

## Current fixture scope

The current fixture foundation is deliberately narrow:

- pure Rust cleanup/scoring tests may use synthetic strings;
- tests must not require Tesseract or a desktop session;
- tests must not write screenshots or OCR output artifacts;
- generated or checked-in fixtures must remain small enough to review directly.

Real OCR accuracy testing remains a separate release task. It should use generated screenshots from synthetic text, not private screenshots, and should keep generated artifacts out of CI uploads unless explicitly reviewed.
