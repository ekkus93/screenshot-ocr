---
name: build-app
description: Build the app (frontend + Tauri/Rust release build), without running lint/format/tests. Use when the user just wants to build or package the app.
---

Run this via the Agent tool with `model: "haiku"` (mechanical build/pass-fail check, not something that needs a stronger model) and `subagent_type: "general-purpose"`. Give the agent this task:

Run these commands from the repo root, in order, and stop at the first failure:

```bash
npm run build
npm run tauri build
```

Note: if `npm run build` fails because dependencies are missing, run `npm ci` first and then retry.

Report back concisely: which commands passed, which failed, and for any failure show only the relevant error output (not the full log). Do not run lint, format, typecheck, or test commands — this check is build only. For lint + test, use `/lint-n-test`; for the full authoritative check, use `/validate`.
