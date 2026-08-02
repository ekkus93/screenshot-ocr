# Screenshot OCR Hosted Baseline — 2026-08-02

**Repository:** `ekkus93/screenshot-ocr`  
**Branch:** `master`  
**Candidate commit:** `214a661c42b029b815947fb49361f79cc679b889`  
**CI run:** `30726015129`  
**Result:** Success

## Proven gates

The following gates passed on the same exact commit using Ubuntu 22.04 hosted runners:

### Repository policy

- required repository files and committed lockfiles are present;
- all first-party Rust, TypeScript, JavaScript, TSX, and JSX files are at or below 800 lines;
- no broad wildcard Tauri capability is present.

### Frontend

- locked dependency installation;
- Prettier formatting check;
- ESLint with `--max-warnings 0`;
- TypeScript typecheck;
- frontend tests;
- Vite production build.

### Rust

- Rust 1.88.0;
- Rustfmt check;
- Clippy across all targets and features with `-D warnings`;
- Rust tests;
- Rust workspace build.

### Debian packaging

The Ubuntu 22.04 package smoke job built, inspected, checksummed, and uploaded:

```text
PACKAGE_FILE=Screenshot OCR_0.1.0_amd64.deb
PACKAGE_NAME=screenshot-ocr
PACKAGE_VERSION=0.1.0
ARCHITECTURE=amd64
INSTALLED_SIZE_KIB=13952
PACKAGE_SHA256=318e59285556d541294c82ae037fbf40e976dad670366148c0b3cf23391fefa9
```

Declared runtime dependencies:

```text
gnome-screenshot
tesseract-ocr
tesseract-ocr-eng
libwebkit2gtk-4.1-0
libgtk-3-0
```

Uploaded artifact evidence:

```text
ARTIFACT_ID=8826461519
ARTIFACT_NAME=screenshot-ocr-deb-214a661c42b029b815947fb49361f79cc679b889
ARTIFACT_ARCHIVE_SIZE_BYTES=4732785
ARTIFACT_ARCHIVE_SHA256=008c033afaac47aea15beb0158bc7b6e8c4686911b2d5d01cc7440d81065822d
ARTIFACT_EXPIRES=2026-10-31
```

## Scope of this evidence

This baseline proves hosted formatting, linting, typechecking, tests, compilation, and `.deb` construction on Ubuntu 22.04. It does **not** prove real GNOME Wayland/X11 screen selection, clipboard behavior, multi-monitor behavior, fractional scaling, package installation, upgrade, removal, or Ubuntu 24.04 runtime behavior. Those remain separate physical-desktop validation gates in `docs/SCREENSHOT_OCR_V0_1_TODO.md`.

## Subsequent work

The XDG Screenshot Portal implementation began only after this baseline was accepted. Portal commits require a new exact-SHA CI and package result and do not inherit this baseline’s acceptance automatically.
