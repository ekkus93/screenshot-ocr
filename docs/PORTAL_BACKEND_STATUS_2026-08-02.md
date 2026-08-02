# XDG Screenshot Portal Backend Status — 2026-08-02

**Repository:** `ekkus93/screenshot-ocr`  
**Branch:** `master`  
**Status:** Implemented and hosted-CI validated; physical desktop validation remains open

## Implemented behavior

The Rust capture layer includes an XDG Screenshot Portal backend using `ashpd` 0.13.13.

The backend:

- probes the Screenshot portal with a bounded timeout;
- requires Screenshot portal interface version 3 or newer;
- requires the portal to advertise the `Area` capture target;
- requests an interactive, modal, single-area selection;
- maps cancellation, permission denial, timeout, unsupported portal versions, and general portal failures to stable application errors;
- accepts only absolute `file:` result URIs;
- rejects missing, empty, oversized, non-regular, and symlink capture results;
- bounds capture-file reads to 20 MiB;
- decodes the image through the existing bounded image pipeline;
- does not log the result URI, path, pixels, or recognized text.

## Backend selection contract

Selection happens before any selector is opened:

- `Auto` uses the portal only when version 3+ and `Area` support are proven;
- `Auto` otherwise falls back to the trusted GNOME screenshot helper;
- explicit `Portal` fails closed when area support is unavailable;
- explicit `Gnome` does not probe or invoke the portal.

There is no post-selector fallback. Once a portal selector has been requested, a portal failure is returned to the caller rather than opening a second unexpected selector.

## Automated tests added

- portal version and area-target requirements;
- backend preference and fallback policy;
- unsupported and non-file URI rejection;
- oversized result rejection;
- symlink result rejection;
- stable public portal error codes.

## Hosted validation evidence

The permanent read-only workflow passed all mandatory jobs on the same exact commit:

```text
COMMIT_SHA=69e409b2f59028550ae996792c9fa54285acb3a0
CI_RUN_ID=30727038039
REPOSITORY_POLICY=success
FRONTEND_QUALITY=success
RUST_QUALITY=success
DEBIAN_PACKAGE_SMOKE=success
```

The Rust job passed:

- Rustfmt;
- Clippy across all targets and features with `-D warnings`;
- Rust tests;
- Rust workspace build.

The frontend job passed formatting, ESLint with zero accepted warnings, TypeScript checking, tests, and the production Vite build.

The Ubuntu 22.04 package job built and inspected:

```text
PACKAGE_FILE=Screenshot OCR_0.1.0_amd64.deb
PACKAGE_NAME=screenshot-ocr
PACKAGE_VERSION=0.1.0
ARCHITECTURE=amd64
INSTALLED_SIZE_KIB=16860
PACKAGE_SHA256=667fc5847225ec1f7f6377f572375a450d32f706b86a4680b5a29347d2914e93
```

Artifact evidence:

```text
ARTIFACT_ID=8826795552
ARTIFACT_NAME=screenshot-ocr-deb-69e409b2f59028550ae996792c9fa54285acb3a0
ARTIFACT_ARCHIVE_SIZE_BYTES=5785858
ARTIFACT_ARCHIVE_SHA256=e49607558f41959772b0db16bf89ba876c6fa8118af0bd0faf54b2403b177da6
ARTIFACT_EXPIRES=2026-10-31T01:25:56Z
```

## Validation boundary

Hosted validation proves formatting, linting, typechecking, tests, compilation, backend policy tests, and Debian package construction. It does not prove the real desktop selector or D-Bus interaction.

Physical Ubuntu 24.04 GNOME Wayland validation remains required for:

- real portal area selection;
- portal cancellation and permission denial;
- multiple monitors;
- fractional scaling;
- clipboard behavior after a portal capture;
- package installation and launcher behavior.

## Remaining portal work

- integrate application-level cancellation with portal requests where the `ashpd` request lifecycle permits explicit closure;
- surface safe portal capability details in diagnostics;
- validate cancellation and permission-denial behavior on a real GNOME session;
- update `docs/SCREENSHOT_OCR_V0_1_TODO.md` only for items backed by the required evidence.
