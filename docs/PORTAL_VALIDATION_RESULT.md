# Portal Backend Hosted Validation

**Trigger commit:** `2d202cf7aebdea8781e8838fbf3b498da469415e`  
**Run:** `30726845124`  
**Runner:** Ubuntu 22.04  
**Rust:** 1.88.0

- `cargo-format`: success
- `cargo-clippy`: success
- `cargo-test`: success
- `cargo-build`: success
- `npm-install`: success
- `frontend-format`: failure (exit 1)

## Sanitized failure output

```text

> screenshot-ocr@0.1.0 format:check
> prettier --check .

Checking formatting...
[[33mwarn[39m] src-tauri/gen/schemas/acl-manifests.json
[[33mwarn[39m] src-tauri/gen/schemas/capabilities.json
[[33mwarn[39m] src-tauri/gen/schemas/desktop-schema.json
[[33mwarn[39m] src-tauri/gen/schemas/linux-schema.json
[[33mwarn[39m] Code style issues found in 4 files. Run Prettier with --write to fix.

```

**Result:** failure at `frontend-format`
