# Screenshot OCR FIX1 Progress — 2026-08-02

**Repository:** `ekkus93/screenshot-ocr`  
**Branch:** `master`  
**FIX1 TODO:** `docs/SCREENSHOT_OCR_FIX1_TODO_2026-08-02.md`  
**Baseline before first FIX1 implementation pass:** `d77b5281975e2773d51afb0ce52d2ff77d966986`  
**Latest source commit after cleanup tranche:** `ee79a7fddd731d3d756033d2a168b36f69c8f919`  
**Status:** implementation tranches in progress; CI/package validation not yet proven in this document

## 1. Scope completed so far

These Ralph passes targeted high-risk FIX1 items that can be implemented from source review without physical Ubuntu desktop access.

Implemented changes:

- moved production controller prop typing out of `src/test` and into `src/app/controllerTypes.ts`;
- removed the obsolete `src/test/types.ts` helper;
- changed capture status copy so the UI no longer falsely remains semantically stuck at selection while OCR continues after selection;
- disabled active-looking Settings controls for unimplemented notification, start-at-login, and close-to-tray behavior;
- made the preserve-whitespace Settings control truthful as always-on/reserved for the current pre-release behavior;
- added Settings-page warning/error regions using accessible alert patterns;
- added frontend tests for Settings save failure visibility, unsaved-value preservation, corrupt-settings warning visibility, and reserved-control disablement;
- added a Rust `SettingsLoadResult` / `SettingsRecoveryWarning` recovery contract;
- changed `get_settings` so corrupt/invalid settings return safe defaults plus a visible safe warning instead of silently returning defaults;
- preserved corrupt-settings quarantine behavior;
- ensured settings recovery warnings serialize without raw settings content or paths;
- changed immediate-copy clipboard failure to return the OCR text with `copied = false` and a `clipboard_write_failed` warning instead of throwing away recognized text;
- added a safe clipboard-warning helper test;
- converted Tesseract language probing from blocking `std::process::Command::output()` to bounded async Tokio process execution with cancellation, stdout limits, no shell, stdin null, stderr null, timeout, and kill/reap behavior;
- passed the capture cancellation token into Tesseract language probing;
- used bounded language probing in diagnostics;
- renamed the misleading `GrayscaleContrast` preprocessing variant to `Grayscale`;
- removed the unused `Upscale3x` enum value for now;
- rechecked generated upscale dimensions before resizing and skipped unsafe upscales;
- added image-pipeline tests for truthful grayscale naming and resize-bound behavior;
- added GNOME capture-directory ownership markers;
- added conservative stale GNOME capture-directory scavenging limited to old `screenshot-ocr-*` directories with an exact content-free ownership marker;
- added tests proving unmarked directories and symlink markers are not scavenged;
- documented the portal lifecycle review and explicitly kept physical portal validation as a separate unreplaced release gate.

## 2. Commit list

```text
33255d831cfa35e370a84df2aacdb11882f0490b  refactor: move controller type out of test namespace
2dbaaf9e49ca15e29f22c5dc08ef1d03fe61b5ad  fix: make capture progress copy truthful
bf9a3e0b9f30ca8b362d069def9192329e1aa723  refactor: remove production dependency on test types
426dbdd961f3f7bdd8dbaa6bdfca1dcd3150f929  fix: make settings page errors and reserved controls truthful
61a46d4b4fe49310faea9852fafc34b4b5ce770e  feat: type safe settings recovery warnings
80d110b3011b62e43507b16db9834e45bd0198e8  feat: return settings load result to frontend
b116abf28616e0b3ff4ddab5efc28d92ca86c389  feat: surface settings recovery warnings in controller
f20ba582bcfb5e1d765efa988d721876157d72fe  feat: return visible corrupt settings recovery state
23a3f11bb757e22f2e9f8acc24175c592d62861b  fix: preserve OCR text on immediate clipboard failure
3997a8593bac26e9d70438a35cc47bdea3a56ea9  fix: make capture language probing cancellable
75c86a19da99daee1b81280035a0ac7733a31e96  fix: bound and cancel Tesseract language probing
a5ad86a2f7c2ca91fd4692f0d19fe6fe143046e2  fix: remove misleading preprocessing variants
cb168f4ad0e67cc5bb444728f3c9aa6b0ca9d6ba  fix: bound generated preprocessing variants
770de3e4e21a7a8a3d9dc9dae702ad2b36819402  test: cover settings recovery and truthful controls
e8abf9e00a43128b4ffb037331487e64a735d3c2  test: avoid ambiguous reserved-control copy lookup
612226f28df07e11deceb8a02ec9ec1f5828ae30  refactor: remove obsolete test controller type helper
ebb58660bb6a3c19169c187e93b181c4cb07c696  fix: mark and scavenge owned GNOME capture directories
ee79a7fddd731d3d756033d2a168b36f69c8f919  docs: record portal lifecycle FIX1 review
```

## 3. FIX1 TODO mapping

### F1.1 — Settings UI truthfulness and Settings-page errors

Implemented in source:

- active notification/start-at-login/close-to-tray controls are disabled with explicit pre-release copy;
- preserve-whitespace is shown as always-on for terminal/code capture rather than as a misleading active setting;
- Settings warnings and errors render directly on the Settings tab;
- save failure preserves unsaved values;
- frontend tests were added for the above.

Remaining before marking the whole milestone complete:

- run frontend format/lint/typecheck/tests/build on the final commit;
- update user/developer docs if the reserved settings behavior needs more explicit documentation;
- decide whether to remove schema fields later or keep them reserved.

### F1.2 — Corrupt settings recovery

Implemented in source:

- `SettingsLoadResult` and `SettingsRecoveryWarning` exist;
- corrupt/invalid settings now produce defaults plus a warning instead of silent success;
- the warning is content-free and path-free;
- corrupt settings quarantine behavior is preserved;
- runtime diagnostics record the stable settings-invalid code;
- frontend displays the warning on the Settings tab;
- Rust and frontend tests were added.

Remaining before marking the whole milestone complete:

- run Rust/frontend CI gates on the final commit;
- inspect serialized DTO output in the built app if needed.

### F1.3 — Immediate-copy clipboard failure recovery

Implemented in source:

- selected Option A: return `OcrResult` with `copied = false` plus warning `clipboard_write_failed`;
- OCR text is preserved in the ordinary result path after immediate-copy clipboard failure;
- the frontend already places any non-copied successful result into the editor and shows warnings;
- safe runtime diagnostics record the clipboard failure code;
- warning helper test added.

Remaining before marking the whole milestone complete:

- add a fuller command-level fake/integration test if feasible;
- add frontend test specifically covering immediate-copy warning recovery;
- run CI gates.

### F1.4 — Capture/OCR progress state

Implemented conservative alternative:

- status text changed from selection-only copy to `Select a region; OCR continues after selection`.

Remaining:

- full Rust stage events were not implemented in this tranche;
- `processing` state is still reserved but not driven by backend stage events;
- stage-event tests remain open.

### F1.5 — Tesseract probe hardening

Implemented in source:

- language probe is async and bounded;
- no shell invocation;
- stdin null;
- stderr null;
- stdout bounded;
- timeout applied;
- child kill/reap used on timeout/cancellation;
- cancellation token passed through capture language probing;
- diagnostics path uses the bounded probe;
- tests added for valid filtering, missing English, and pre-cancelled probe.

Remaining:

- add explicit hanging-helper timeout test if CI runtime budget allows;
- add oversized stdout test;
- decide and document bounded `tesseract --version` as implemented or deferred.

### F1.6 — Image preprocessing limits and naming

Implemented in source:

- generated 2x dimensions are checked with checked arithmetic;
- scaled dimensions are revalidated against max dimensions/pixels before resize;
- unsafe upscale is skipped;
- `GrayscaleContrast` renamed to `Grayscale`;
- unused `Upscale3x` removed/deferred;
- tests added for grayscale naming and resize-bound policy.

Remaining:

- update any user/developer docs that still mention the old variant names;
- add broader boundary tests if desired;
- run CI gates.

### F1.7 — GNOME temp ownership and stale cleanup

Implemented in source:

- new capture directories receive `.screenshot-ocr-owned` marker files with static content-free text;
- marker files are created with private permissions where supported;
- if permission or marker creation fails, the new capture directory is removed before returning failure;
- startup-time capture-directory creation opportunistically scavenges stale owned directories from the same runtime/temp base;
- scavenging only considers names beginning `screenshot-ocr-`;
- scavenging only removes directories that contain the exact valid regular-file marker;
- symlink markers are rejected;
- unmarked directories are left untouched;
- tests cover marker requirement, unowned directory preservation, and symlink-marker rejection.

Remaining:

- run Rustfmt, Clippy, and Rust tests;
- physically inspect runtime directories after success/cancel/failure on Ubuntu GNOME;
- decide whether to expose scavenged-count diagnostics later.

### F1.8 — Portal lifecycle review

Implemented in documentation:

- created `docs/SCREENSHOT_OCR_FIX1_PORTAL_LIFECYCLE_REVIEW_2026-08-02.md`;
- recorded why FIX1 does not add a source-level request-close hook for the current `ashpd` request usage;
- recorded that physical portal UI/result lifecycle validation remains mandatory and unclaimed.

Remaining:

- physical GNOME portal validation;
- targeted source change only if physical testing finds stale prompts/artifacts.

### F1.9 — Frontend production/test boundary

Implemented in source:

- production components import `AppController` from `src/app/controllerTypes.ts`;
- obsolete `src/test/types.ts` was removed.

Remaining:

- run frontend lint/typecheck/tests/build.

## 4. Validation status

Not yet proven in this document:

- frontend formatting;
- ESLint;
- TypeScript checking;
- frontend tests;
- production frontend build;
- Rustfmt;
- Clippy with `-D warnings`;
- Rust tests;
- Rust workspace build;
- Debian package smoke;
- package artifact checksum.

Connector checks performed:

- combined commit status for `fe0ef2593490020ad76faf0ca0647f273e39d35e` returned no status entries;
- the GitHub connector did not return workflow runs for `fe0ef2593490020ad76faf0ca0647f273e39d35e`;
- compare from `d77b5281975e2773d51afb0ce52d2ff77d966986` to `612226f28df07e11deceb8a02ec9ec1f5828ae30` showed 16 commits touching the expected Rust/frontend files before the cleanup tranche;
- latest cleanup tranche source/doc commits are recorded above.

Local validation was not run from this environment because the local container could not resolve `github.com` to clone the repository. Do not treat this status file as CI evidence.

## 5. Next Ralph pass

Recommended next actions:

1. Inspect the final CI run for `ee79a7fddd731d3d756033d2a168b36f69c8f919` or later.
2. Fix any format, lint, typecheck, test, or build failures.
3. Add missing immediate-copy frontend test and stronger Tesseract probe timeout/stdout tests.
4. Update `docs/SCREENSHOT_OCR_FIX1_TODO_2026-08-02.md` checkboxes only after CI proves the relevant tasks.
5. Continue with F1.11 content-leakage guard and F1.12 public error-code reconciliation.
