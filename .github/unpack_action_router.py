from __future__ import annotations

import base64
import json
from pathlib import Path

WORKFLOW = Path(".github/workflows/implement-action-router.yml")
MARKER = "payload = json.loads(r'''"
EXPECTED_PATHS = {
    "docs/adr/0002-cross-platform-activation-and-cancellation.md",
    "src-tauri/Cargo.toml",
    "src-tauri/src/actions.rs",
    "src-tauri/src/app.rs",
    "src-tauri/src/commands.rs",
    "src-tauri/src/desktop.rs",
    "src-tauri/src/diagnostics.rs",
    "src-tauri/src/lib.rs",
    "src-tauri/src/models.rs",
    "src-tauri/src/state.rs",
    "src/app/App.test.tsx",
    "src/app/useAppController.ts",
    "src/lib/tauri.ts",
    "src/lib/types.ts",
}

workflow = WORKFLOW.read_text(encoding="utf-8")
if workflow.count(MARKER) != 1:
    raise SystemExit("reviewed payload marker is missing or ambiguous")

payload_source = workflow.split(MARKER, 1)[1].lstrip()
payload, _ = json.JSONDecoder().raw_decode(payload_source)
actual_paths = set(payload)
if actual_paths != EXPECTED_PATHS:
    extra = sorted(actual_paths - EXPECTED_PATHS)
    missing = sorted(EXPECTED_PATHS - actual_paths)
    raise SystemExit(
        f"reviewed payload path set changed; extra={extra}; missing={missing}"
    )

for relative, value in payload.items():
    path = Path(relative)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(base64.b64decode(value, validate=True))
