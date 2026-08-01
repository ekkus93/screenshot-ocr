# Screenshot OCR

Screenshot OCR is a privacy-first Linux desktop utility for selecting visible screen text, recognizing it locally with Tesseract, and copying it to the clipboard. It is optimized for terminal output, source code, and developer punctuation.

> **Status:** pre-release. Automated development is active; the mandatory Ubuntu 22.04/24.04 Wayland/X11 validation matrix is not yet complete.

## Initial platform scope

- Ubuntu 22.04 LTS, GNOME Wayland and X11
- Ubuntu 24.04 LTS, GNOME Wayland and X11
- English OCR with Tesseract 5
- GNOME screenshot helper compatibility backend
- `.deb` release package built on Ubuntu 22.04

Other distributions and desktop environments are unverified.

## Architecture

- React, TypeScript, Vite, and Tailwind CSS frontend
- Tauri 2 desktop shell
- Rust application core for capture, image processing, OCR, settings, diagnostics, clipboard policy, and privacy-sensitive behavior
- Local-only OCR; no capture or recognized text is sent over the network

See:

- [v0.1 specification](docs/SCREENSHOT_OCR_V0_1_SPEC.md)
- [implementation TODO](docs/SCREENSHOT_OCR_V0_1_TODO.md)
- [architecture](docs/ARCHITECTURE.md)
- [privacy](docs/PRIVACY.md)
- [threat model](docs/THREAT_MODEL.md)
- [CI status bridge](docs/CI_STATUS_BRIDGE.md)

## Development prerequisites

Ubuntu 22.04 build packages:

```bash
sudo apt-get update
sudo apt-get install --yes \
  build-essential curl file libayatana-appindicator3-dev librsvg2-dev \
  libssl-dev libwebkit2gtk-4.1-dev libxdo-dev pkg-config wget
```

Runtime packages for the compatibility path:

```bash
sudo apt-get install --yes gnome-screenshot tesseract-ocr tesseract-ocr-eng
```

The project pins Node and Rust versions. Run `scripts/check-dev-tools.sh` to inspect prerequisites without modifying the host.

## Validation

```bash
npm ci
npm run validate
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --workspace --all-targets --all-features --locked -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-features --locked
npm run tauri build
```

Hosted CI on Ubuntu 22.04 is authoritative for compilation and linting. Physical desktop validation remains a separate release gate.

## Shortcut and startup capture

The documented default is `Super+Shift+O`. On GNOME Wayland, create a custom keyboard shortcut that runs:

```text
screenshot-ocr capture
```

The binary recognizes only the exact first argument `capture`. On a fresh launch, that argument is consumed once by the Rust backend, then the React application starts the normal configured capture workflow after settings and diagnostics load. Re-rendering the frontend cannot replay the startup request.

The application does not modify GNOME settings automatically.

## License

MIT. See [LICENSE](LICENSE).
