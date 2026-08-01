# Development and validation

Run `scripts/check-dev-tools.sh` for a non-mutating prerequisite check. Hosted CI uses Ubuntu 22.04 and runs frozen frontend installation, Prettier, ESLint with zero warnings, TypeScript strict checks, Vitest, production frontend build, Rustfmt, Clippy with warnings denied, Rust tests, Rust build, and Tauri `.deb` packaging.

Manual system tests must use synthetic terminal content and record exact package checksum, Ubuntu/GNOME/session versions, portal and Tesseract versions, monitor/scaling configuration, selected backend, and results in `docs/validation/`.

Settings schema and application versions use semantic versioning. A schema migration must be explicit, tested, and preserve a recoverable copy of invalid data.
