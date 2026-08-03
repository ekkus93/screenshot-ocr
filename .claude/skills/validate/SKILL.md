---
name: validate
description: Run the full authoritative validation sequence for screenshot-ocr (frontend + Rust + packaging build), matching what CI actually enforces. Use before declaring any change to this repo done, or when the user asks to validate, check, or verify the whole project.
---

`npm run validate` only covers the frontend half of this repo's checks. Run the complete sequence from the README, in order, and stop at the first failure:

```bash
npm ci
npm run validate   # format:check && lint && typecheck && test && build
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --workspace --all-targets --all-features --locked -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-features --locked
npm run tauri build
```

Notes:

- `cargo fmt --check` and `cargo clippy -D warnings` both treat any diff/warning as failure — never `--allow` or reformat-and-ignore, fix the underlying code.
- If `check-content-leakage.py` or the `repository-policy` CI job would fail (debug logging left in first-party code, files over 800 lines, wildcard Tauri permissions), fix those before re-running rather than skipping ahead.
- `npm run tauri build` is slow (full Tauri/Rust release build) — only skip it if the user explicitly asks for a fast frontend-only check, and say so.
- Passing this sequence locally does not replace physical Wayland/X11 validation (`docs/validation/`) — mention that gate still applies for release-readiness questions.
