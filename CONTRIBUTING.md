# Contributing

Work is performed directly on `master` unless the repository owner explicitly requests another branch. Do not create pull requests or branches merely for automation convenience.

## Quality policy

- Every warning or lint finding in first-party code is a defect.
- Do not suppress, downgrade, or hide first-party warnings.
- Third-party and vendored-code warnings are outside this policy.
- Keep source files below 800 lines; split them before they exceed the limit.
- Never log captured pixels, OCR text, clipboard content, or temporary paths.
- Add deterministic tests for every behavioral fix.
- Do not substitute hosted CI for physical Wayland/X11 validation.

Run the commands documented in `README.md` before publishing a change.
