# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Privacy-first Linux desktop utility (Tauri 2 + React/TS frontend, Rust backend) that lets a user select visible screen text, recognize it locally with Tesseract, and copy it to the clipboard. No network calls, local-only OCR. Pre-release; Ubuntu 22.04/24.04 GNOME Wayland/X11 validation is incomplete (see `docs/validation/`).

## Validation

The README's exact sequence is authoritative — run it in full before considering work done, not just the npm parts:

```bash
npm ci
npm run validate   # format:check && lint && typecheck && test && build
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --workspace --all-targets --all-features --locked -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-features --locked
npm run tauri build
```

Hosted CI on Ubuntu 22.04 is authoritative for compilation/linting; it is **not** a substitute for physical Wayland/X11 validation (a separate release gate).

## Hard rules (CI-enforced, do not weaken)

- **Zero-warning policy**: every lint/warning in first-party code is a defect. Never suppress, downgrade, or `#[allow]`/`eslint-disable` around a first-party warning. `cargo clippy` runs with `-D warnings`; `eslint` runs with `--max-warnings 0`.
- **No content leakage**: never add `dbg!`, `println!`, `eprintln!` (Rust) or any `console.*` call (TS/JS/TSX/JSX) to first-party source, even as temporary debug scaffolding — `scripts/check-content-leakage.py` fails CI on these because captured pixels/OCR text/clipboard content/temp paths must never reach logs.
- **800-line cap** per first-party `.rs/.ts/.tsx/.js/.jsx` file, enforced by CI. Split before hitting it.
- **Synthetic fixtures only**: tests must never contain real screenshots, real OCR output, real clipboard text, real temp paths, or real portal URIs/helper stdio. Use synthetic markers (e.g. `SYNTHETIC_OCR_FIXTURE_...`, `SYNTHETIC_SECRET_...`) per `docs/OCR_SYNTHETIC_FIXTURES.md`.
- **No wildcard Tauri permissions**: `src-tauri/capabilities/default.json` must never contain a `*` in `permissions` — CI greps for this.
- **CLI contract is fixed**: the binary's first argument must be exactly one of `capture cancel toggle show quit`. Never add implicit-capture behavior for unrecognized args.
- **`ashpd` is exact-pinned** (`=0.13.13`) in `src-tauri/Cargo.toml` — don't bump it casually; it's tied to portal API stability.

## Architecture conventions

- Internal Rust errors (`AppError`) never serialize directly to the frontend — they're mapped through `From<AppError> for PublicError` into a hand-written `message`/`guidance`/`retryable` triple with no dynamic/interpolated content, so paths/secrets can't leak across the Tauri IPC boundary. Follow this pattern for new error variants.
- DTOs crossing the Tauri boundary use `#[serde(rename_all = "camelCase")]`; the error-code enum uses `snake_case`. Settings deserialization uses `deny_unknown_fields`.
- Frontend calls Tauri only through typed wrappers in `src/lib/tauri.ts` (one function per backend command) — don't call `invoke` ad hoc from components.
- `src/app/useAppController.ts` centralizes app state/business logic consumed by `App.tsx`.
- Production code (`src/app`, `src/features`, `src/lib`) must never import from `src/test` (test-only helpers).
- Reserved/inactive settings (start-at-login, close-to-tray, notify-after-copy) must never appear as active in the UI — disable with explicit copy or omit the control entirely until the backend implements it.
- Portal capture (`src-tauri/src/capture/portal.rs`) is intentionally not enabled — it's a review boundary only until explicit area-target capture is proven.

## Repo etiquette

- Work happens directly on `master`. Do not create pull requests or branches merely for automation convenience — only do so if explicitly asked.
- For deeper context, see `docs/ARCHITECTURE.md`, `docs/THREAT_MODEL.md` (protected assets, mitigations, residual risks), `docs/PRIVACY.md`, and `SECURITY.md` (private disclosure process — never attach real screenshots/credentials/OCR output to public issues).
- Current work priorities live in `docs/SCREENSHOT_OCR_V0_1_TODO.md` and the latest `docs/SCREENSHOT_OCR_FIX1_*` status docs — check these before starting new work.
