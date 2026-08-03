---
name: lint-n-test
description: Lint the frontend and backend, then run all tests, without doing a full validate/build. Use when the user asks to lint and test, or just wants fast pass/fail feedback rather than the full /validate sequence.
---

Run this via the Agent tool with `model: "haiku"` (this is a mechanical pass/fail check, not something that needs a stronger model) and `subagent_type: "general-purpose"`. Give the agent this task:

Run these commands from the repo root, in order, and stop at the first failure:

```bash
npm run lint
npm run test
cargo clippy --manifest-path src-tauri/Cargo.toml --workspace --all-targets --all-features --locked -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-features --locked
```

Report back concisely: which commands passed, which failed, and the relevant error output for any failure (not the full log). Do not run `format:check`, `typecheck`, or any build step — this check is lint + test only. For the full authoritative check (including build), use the `/validate` skill instead.
