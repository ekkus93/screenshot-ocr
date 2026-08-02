# Screenshot OCR FIX1 Evidence Reconciliation — 2026-08-02

**Repository:** `ekkus93/screenshot-ocr`  
**Branch:** `master`  
**Baseline reviewed commit:** `b45fc89027dddef981332dbde976122cffa70050`  
**FIX1 implementation baseline:** `d77b5281975e2773d51afb0ce52d2ff77d966986`  
**Final automated-validation source commit:** `a85002390f60c918a01bffbbde7431ca2c49e0bc`  
**CI run:** `30741571581`  
**CI result:** `success`  
**Status issue:** `https://github.com/ekkus93/screenshot-ocr/issues/1`

## 1. Executive status

The FIX1 source tranche reached a green hosted CI state on commit
`a85002390f60c918a01bffbbde7431ca2c49e0bc`.

This is **automated evidence only**. It does not replace the required physical
Ubuntu desktop validation matrix for Ubuntu 22.04 / 24.04, GNOME Wayland / X11,
real clipboard behavior, tray behavior, shortcut behavior, portal prompts,
multi-monitor/scaling, package install/remove behavior, or OCR accuracy.

## 2. Hosted CI evidence

The authoritative CI status bridge reported the following for run `30741571581`:

```text
Status: completed
Conclusion: success
Run: 30741571581
Commit: a85002390f60c918a01bffbbde7431ca2c49e0bc
Branch/event: master / push
Jobs: 4 completed, 0 abnormal, 0 running, 4 visible
Artifacts: 1 available
Observed: 2026-08-02T09:30:50.639905Z
```

Successful jobs:

```text
Repository policy: success
Frontend quality gates: success
Rust quality gates: success
Debian package smoke: success
```

Hosted CI therefore covered:

```text
npm ci
npm run format:check
npm run lint
npm run typecheck
npm run test
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --workspace --all-targets --all-features --locked -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-features --locked
cargo build / Rust workspace build path used by the workflow
npm run tauri -- build --bundles deb, through the Debian package smoke job
```

## 3. Package artifact evidence

GitHub Actions artifact metadata:

```text
Artifact name: screenshot-ocr-deb-a85002390f60c918a01bffbbde7431ca2c49e0bc
Artifact ID: 8831559732
Artifact size: 6,329,707 bytes
Artifact created: 2026-08-02T09:30:42Z
Artifact expires: 2026-10-31T09:20:25Z
GitHub artifact digest: sha256:3b4aebc848cebe9d41d152d76384e40fff4a68782b88cc38e9b1b2471a4fe226
```

Downloaded artifact verification in this reconciliation pass:

```text
Downloaded ZIP: screenshot-ocr-deb-a85002390f60c918a01bffbbde7431ca2c49e0bc.zip
Downloaded ZIP SHA-256: 3b4aebc848cebe9d41d152d76384e40fff4a68782b88cc38e9b1b2471a4fe226
```

The downloaded ZIP SHA-256 matches the GitHub artifact digest.

ZIP contents:

```text
src-tauri/target/release/bundle/deb/Screenshot OCR_0.1.0_amd64.deb  6,360,804 bytes
package.sha256                                                                  133 bytes
```

`package.sha256` content:

```text
24fcdac40d8b3c502177472875548d2388e95ae30aef56c0a4fa81aeea3dfc7e  src-tauri/target/release/bundle/deb/Screenshot OCR_0.1.0_amd64.deb
```

Local SHA-256 of the extracted Debian package matched `package.sha256`:

```text
24fcdac40d8b3c502177472875548d2388e95ae30aef56c0a4fa81aeea3dfc7e  Screenshot OCR_0.1.0_amd64.deb
```

Debian package metadata inspected with `dpkg-deb`:

```text
Package: screenshot-ocr
Version: 0.1.0
Architecture: amd64
Installed-Size: 18347
Maintainer: Phillip Chin
Depends: gnome-screenshot, tesseract-ocr, tesseract-ocr-eng, libayatana-appindicator3-1, libwebkit2gtk-4.1-0, libgtk-3-0
Description: Copy visible screen text with local OCR
 Select a screen region, recognize text locally with Tesseract, and copy it to the clipboard.
```

## 4. Commit evidence after the previous progress document

The previous progress document ended before the formatting and final Clippy repair
loop completed. The following additional commits are part of the final green
source state:

```text
f728afaec324b1642af3abb786c0b23e3c43450a  style: apply rustfmt to GNOME capture cleanup
c2634ad8e22d202fd5879bf112bac9a2f7916824  style: apply rustfmt to command module
f32cdbdc4f08372023889eeeae8b1ddb4d7f3a3a  style: apply rustfmt to image pipeline tests
f3c1968ec76b07b8e9119625976be4b38f923bd6  style: apply rustfmt to Tesseract tests
5c3e8d1723532e39a588822b3bd2f8b7ec3a08ca  style: apply rustfmt to settings recovery
4f7b78d988b4f58459d12169519bdac714b084aa  style: apply Prettier to app tests
b2cdb0662673b0a1f68b3085e0c33e26716fabc1  style: apply Prettier to settings panel
a85002390f60c918a01bffbbde7431ca2c49e0bc  fix: remove unused clipboard unavailable error variant
```

Run `30739777215` failed on formatting. Run `30740314474` proved formatting had
been fixed but failed Clippy because `AppError::ClipboardUnavailable` was dead
code under `-D warnings`. Commit `a85002390f60c918a01bffbbde7431ca2c49e0bc`
removed that unused path, and run `30741571581` passed.

## 5. Reconciled FIX1 milestone status

### F1.0 — Baseline and safety guardrails

Automated evidence is green on commit `a85002390f60c918a01bffbbde7431ca2c49e0bc`.
No evidence in this pass showed a weakened no-shell, single-capture, no-history,
Tauri-capability, GNOME fallback, or fail-closed portal policy.

### F1.1 — Settings UI truthfulness and Settings-page errors

Reconciled as implemented and automatically tested for this tranche:

- unimplemented notification/start-at-login/close-to-tray controls are disabled
  rather than presented as active features;
- preserve-whitespace is represented truthfully for the current pre-release
  behavior;
- Settings warnings/errors are visible on the Settings tab;
- frontend tests passed under hosted CI.

### F1.2 — Corrupt settings recovery

Reconciled as implemented and automatically tested:

- backend returns settings plus a safe warning result;
- corrupt settings recovery is visible to the frontend;
- warnings are content-free/path-free;
- Rust and frontend tests passed under hosted CI.

### F1.3 — Immediate-copy clipboard failure recovery

Reconciled as implemented and automatically tested for the selected design:

- OCR success plus immediate-copy failure returns recognized text with
  `copied = false` and a `clipboard_write_failed` warning;
- frontend preserves text and allows retry;
- tests passed under hosted CI.

### F1.4 — Capture/OCR progress state

Only the conservative alternative is reconciled as implemented:

- UI copy no longer claims the app remains only in selection while OCR continues;
- full Rust stage-event DTOs and frontend stage-event routing were not
  implemented in FIX1 and remain open.

### F1.5 — Tesseract probe hardening

Reconciled as implemented and automatically tested for language probing:

- language probing is bounded, async, cancellation-aware, no-shell, stdin-null,
  stderr-null, stdout-limited, timeout-protected, and kill/reap protected;
- diagnostics use the bounded language probe;
- hosted CI passed the associated Rust tests.

Still open: bounded `tesseract --version` was not separately implemented or
proven.

### F1.6 — Image preprocessing limits and naming

Reconciled as implemented and automatically tested for the addressed scope:

- generated resize dimensions are checked with bounded arithmetic;
- unsafe upscales are skipped;
- `GrayscaleContrast` was renamed to `Grayscale`;
- unused `Upscale3x` was removed/deferred;
- hosted CI passed.

### F1.7 — GNOME temp ownership and stale cleanup

Reconciled as implemented and automatically tested for source-level behavior:

- capture directories use a content-free ownership marker;
- scavenging is constrained to matching `screenshot-ocr-*` directories with the
  exact regular-file marker;
- unmarked directories and symlink-marker cases are protected by tests;
- hosted CI passed.

Still open: physical runtime-directory inspection on Ubuntu GNOME remains
unproven.

### F1.8 — Portal lifecycle review

Reconciled as documented:

- portal lifecycle review exists in
  `docs/SCREENSHOT_OCR_FIX1_PORTAL_LIFECYCLE_REVIEW_2026-08-02.md`;
- no claim is made that physical portal prompt/result lifecycle validation has
  been completed.

### F1.9 — Frontend production/test boundary

Reconciled as implemented and automatically tested:

- production controller typing was moved into `src/app/controllerTypes.ts`;
- obsolete `src/test/types.ts` was removed;
- frontend typecheck/build passed under hosted CI.

### F1.10 — Tray, shortcut, startup truthfulness

Partially covered only where F1.1 disabled startup/close-to-tray UI overpromise.
The broader tray/shortcut/startup truthfulness checklist remains open.

### F1.11 — Content-leakage guard and OCR fixture foundation

Partially reconciled:

- source-level leakage guard exists and is run by repository-policy CI;
- repository-policy CI passed.

Still open:

- richer synthetic OCR fixture foundation;
- log-capture/fault-injection leakage tests;
- real OCR fixture/accuracy evidence.

### F1.12 — Public error-code and CI/security reconciliation

Reconciled final state:

- `clipboard_write_failed` is the proven recoverable clipboard failure code;
- a separate `clipboard_unavailable` public error path is **not** present in the
  final green source state because the attempted `AppError::ClipboardUnavailable`
  path was unused and removed to satisfy `-D warnings`;
- action-SHA pinning and dependency/security tooling decisions remain open.

### F1.13 — Documentation and authoritative TODO reconciliation

This file records the evidence reconciliation. It should be treated as the
source for updating checklist boxes and status notes without overclaiming.

### F1.14 — Final validation and evidence

Automated hosted CI validation is green for commit
`a85002390f60c918a01bffbbde7431ca2c49e0bc` on run `30741571581`.
Package artifact and checksums are recorded above.

Manual validation remains unclaimed.

## 6. Remaining release blockers after this pass

Do not mark these complete without separate evidence:

- Ubuntu 22.04 GNOME Wayland physical validation;
- Ubuntu 22.04 GNOME X11 physical validation;
- Ubuntu 24.04 GNOME Wayland physical validation;
- Ubuntu 24.04 GNOME X11 physical validation;
- real clipboard paste/ownership validation;
- real tray behavior validation;
- real GNOME custom shortcut validation;
- real portal permission/cancellation validation;
- multi-monitor validation;
- 125%, 150%, and 200% scaling validation;
- package install/upgrade/reinstall/removal validation;
- full OCR regression and accuracy evidence;
- dependency/license/security review;
- final v0.1 implementation report;
- release tag/signoff.

## 7. Recommended next implementation tranche

Recommended next source/docs work after this evidence pass:

1. F1.10 tray/shortcut/startup truthfulness.
2. F1.11 synthetic OCR fixture foundation and richer content-leakage tests.
3. F1.12 action-SHA pinning and dependency/security-tooling decision.
4. Physical Ubuntu validation prep and checklist execution.
