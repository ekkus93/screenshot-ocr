# Privacy

Screenshot OCR performs OCR locally. It does not upload screenshots or recognized text and does not require network access for capture or OCR.

The GNOME compatibility backend temporarily asks `gnome-screenshot` to write one image inside a private, randomly named runtime directory. The image is read, deleted immediately, decoded in application memory, and the directory is removed. OCR preprocessing stays in memory, and PNG data is sent to Tesseract through standard input.

GNOME capture directories created by the application contain a static `.screenshot-ocr-owned` marker so startup cleanup can distinguish application-owned stale directories from unrelated files. The marker is content-free and must not contain usernames, screenshot text, paths, command arguments, timestamps, or OCR output.

Version 0.1 has no persistent OCR history. Settings contain preferences only. Corrupt settings are quarantined and recovered with defaults through a safe warning. That warning is intended to include only stable code, message, guidance, and recovery status; it must not include filesystem paths, raw settings JSON, or user content.

Diagnostics include application version, operating-system/desktop/session labels, dependency availability, installed OCR language codes, shortcut/tray status, and safe counters. Diagnostics exclude screenshots, recognized text, clipboard contents, executable paths, temporary paths, portal result URIs, raw helper stderr, and raw helper stdout.

The clipboard changes only after an explicit copy action in preview mode or after successful non-empty OCR in immediate-copy mode. Cancellation, empty output, and OCR failure must leave the clipboard unchanged. If OCR succeeds but an immediate clipboard write fails, the recognized text is returned to the active UI for review/retry with `copied = false`; the app must not claim that the text was copied.

Automated OCR cleanup fixtures are synthetic text only. Private screenshots, real OCR output, clipboard text, temporary paths, portal result URIs, and helper stdout/stderr from a real machine must not be committed as test fixtures or uploaded as CI artifacts.

System components involved are GNOME screenshot facilities, Tesseract, the desktop clipboard, and Tauri/WebKitGTK. Their operating-system behavior remains outside the application's direct control and is covered by the required platform validation matrix.
