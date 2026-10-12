#!/usr/bin/env python3
"""Content-free, reproducible inventory of platform assumptions in product sources.

The inventory reports only repository-relative paths, line numbers, categories,
and architectural owner groups. It is not a static correctness proof.
"""
from __future__ import annotations

import argparse
import json
import re
from pathlib import Path

SUFFIXES = {".rs", ".ts", ".tsx", ".js", ".jsx", ".ps1", ".psm1", ".py", ".toml", ".json", ".yml", ".yaml", ".sh", ".cmd", ".bat", ".nsi", ".nsh", ".desktop", ".c", ".h", ".cpp", ".hpp", ".rc", ".manifest", ".wxs", ".wxi"}
IGNORED = {"node_modules", "target", "dist", "__pycache__", ".git", "coverage"}
PATTERNS = {
    "linux_gnome": r"(?i)\blinux\b|\bgnome\b|\bwayland\b|\bx11\b|LinuxPlatform|GnomeScreenshot",
    "windows_native": r"(?i)\bwindows\b|WindowsPlatform|\bWin32\b|SetThreadDpiAwarenessContext|\bHKEY_[A-Z_]+\b|\bReg(?:Open|Query|Get|Set|Close)Key",
    "windows_environment": r"(?i)\b(?:APPDATA|LOCALAPPDATA|USERPROFILE|PROGRAMFILES|SYSTEMROOT|WINDIR)\b",
    "portal_ashpd": r"(?i)\bportal\b|\bashpd\b|XDG_",
    "desktop_environment": r"DISPLAY|WAYLAND_DISPLAY|XDG_|GNOME_|DESKTOP_SESSION|GDMSESSION",
    "executable_path": r"(?i)tesseract|gnome-screenshot|/usr/(?:local/)?bin/|\.exe\b|[A-Z]:\\",
    "shortcut": r"(?i)shortcut|Control\+Shift\+O|Super\+Shift\+O",
    "target_conditional": r"(?i)\btarget_(?:os|family|env|vendor|arch)\b|#\s*\[\s*cfg(?:_attr)?\b|\bcfg!\s*\(|\bcfg\s*\(\s*(?:windows|unix)\b|\bstd\s*::\s*os\s*::\s*(?:unix|linux|windows)\b",
    "native_error": r"(?i)GetLastError|HRESULT|last_os_error|CapturePermissionDenied|CaptureTimedOut|CaptureProcessFailed|CaptureResultUriInvalid",
    "platform_resource": r"resources/ocr|windows-ocr-runtime|tessdata|app_config_dir|resource_dir|\.desktop\b",
    "native_binding": r"(?i)windows_sys|windows-sys|windows\s*::\s*Win32|Win32_|Win32::|ashpd::|winapi\s*::|winreg\s*::|windows_registry",
    "linux_bundle": r"gnome-screenshot|tesseract-ocr-eng|libwebkit2gtk|libayatana|libgtk-3",
    "bundle_target": r"(?i)\b(?:deb|nsis|msi|appimage)\b|\.desktop\b|webviewInstallMode",
}
COMPILED = {name: re.compile(regex) for name, regex in PATTERNS.items()}


def owner(path: str) -> str:
    if path.startswith("src-tauri/src/platform/linux/"):
        return "linux_adapter"
    if path.startswith("src-tauri/src/platform/windows/"):
        return "windows_adapter"
    if path == "src-tauri/src/platform.rs":
        return "platform_composition"
    if path.startswith("src-tauri/src/platform/"):
        return "platform_contract"
    if path.startswith("src-tauri/src/"):
        return "shared_core_or_desktop"
    if path.startswith("src/"):
        return "shared_frontend"
    if path.startswith("scripts/"):
        return "build_or_validation"
    if path.startswith(".github/workflows/"):
        return "ci"
    if path.endswith("Cargo.toml"):
        return "cargo_manifest"
    return "bundle_or_package_metadata"


def is_unlinked_source(root: Path, path: Path) -> bool:
    """Reject symlinked files and any symlinked ancestor under the checkout."""
    relative = path.relative_to(root)
    return all(
        not root.joinpath(*relative.parts[:depth]).is_symlink()
        for depth in range(1, len(relative.parts) + 1)
    )


def candidates(root: Path) -> list[Path]:
    # Do not silently scan a caller-supplied checkout root redirected elsewhere.
    # In CI this fails the inventory gate rather than trusting external content.
    absolute_root = root.absolute()
    if any(part.is_symlink() for part in (absolute_root, *absolute_root.parents)):
        raise ValueError("platform inventory source root must not be a symlink")
    all_paths: list[Path] = []
    # Scan the entire Tauri tree, not just Rust modules: installer hooks,
    # capability manifests, and target-specific bundle settings also encode
    # OS assumptions. Generated build output remains excluded below.
    for prefix in ("src-tauri", "src", "scripts", ".github/workflows", ".github/actions"):
        base = root / prefix
        if not is_unlinked_source(root, base):
            raise ValueError("platform inventory source tree traverses a symbolic link")
        if base.is_dir():
            all_paths.extend(
                p for p in base.rglob("*")
                if p.is_file() and is_unlinked_source(root, p) and p.suffix in SUFFIXES
                and not any(part in IGNORED for part in p.relative_to(root).parts)
            )
    # Root-level tooling (Vite, Playwright, TypeScript, packaging) may encode
    # platform assumptions outside src/. Do not inventory dependency lockfiles:
    # generated third-party text is not first-party implementation evidence.
    ignored_root_files = {"package-lock.json", "pnpm-lock.yaml", "bun.lockb"}
    all_paths.extend(
        p for p in root.iterdir()
        if p.is_file() and p.suffix in SUFFIXES
        and p.name not in ignored_root_files
        and not p.name.endswith(".lock")
        and is_unlinked_source(root, p)
    )
    return sorted(set(all_paths))


def inventory(root: Path) -> dict:
    records: list[dict] = []
    counts = {key: 0 for key in PATTERNS}
    for path in candidates(root):
        hits: dict[str, list[int]] = {}
        for line_num, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
            for category, regex in COMPILED.items():
                if regex.search(line):
                    hits.setdefault(category, []).append(line_num)
                    counts[category] += 1
        if hits:
            relative = path.relative_to(root).as_posix()
            records.append({"path": relative, "owner": owner(relative), "lines": hits})
    return {
        "schema_version": 1,
        "scope": "product, frontend, scripts, workflows, and build manifests; documentation and lockfiles excluded",
        "counts": counts,
        "entries": records,
        "qualification": "keyword inventory only; requires human boundary review",
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path("."))
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = inventory(args.root)
    if args.output:
        args.output.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print("Platform inventory: %d source files with matches." % len(result["entries"]))
    for category, count in sorted(result["counts"].items()):
        print("%s: %d source lines" % (category, count))
    print("Keyword evidence only; it does not establish platform acceptance.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
