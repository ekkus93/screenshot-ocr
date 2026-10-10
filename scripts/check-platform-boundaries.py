#!/usr/bin/env python3
"""Enforce the portable-core/platform-adapter dependency boundary."""

from __future__ import annotations

from pathlib import Path

RUST_ROOT = Path("src-tauri/src")
PLATFORM_ROOT = RUST_ROOT / "platform"

FORBIDDEN_PORTABLE_PATTERNS = {
    "crate::platform::linux": "portable code imports the Linux adapter",
    "crate::platform::windows": "portable code imports the Windows adapter",
    "GnomeScreenshotBackend": "portable code names the concrete GNOME capture backend",
    "PortalScreenshotBackend": "portable code names the concrete portal capture backend",
    "ashpd::": "portable code imports the Linux portal client",
    'std::env::var("XDG_': "portable code reads a Linux desktop environment variable",
    'std::env::var_os("XDG_': "portable code reads a Linux desktop environment variable",
    'std::env::var("WAYLAND_DISPLAY': "portable code reads a Wayland environment variable",
    'std::env::var("DISPLAY': "portable code reads an X11 environment variable",
    '"/usr/bin"': "portable code hard-codes a Linux executable directory",
    'std::env::var_os("WAYLAND_DISPLAY': "portable code reads a Wayland environment variable",
    'std::env::var_os("DISPLAY': "portable code reads an X11 environment variable",
    'std::env::var("GNOME_': "portable code reads a GNOME environment variable",
    'std::env::var_os("GNOME_': "portable code reads a GNOME environment variable",
    "windows::Win32": "portable code imports a concrete Windows API",
    "windows_sys::Win32": "portable code imports a concrete Windows API",
}

ALLOWED_NON_PLATFORM_FILES = {
    # The composition root is intentionally the single place that selects the
    # current target-specific platform service.
    RUST_ROOT / "platform.rs",
}


def is_platform_file(path: Path) -> bool:
    return PLATFORM_ROOT in path.parents or path == PLATFORM_ROOT


def main() -> int:
    problems: list[str] = []
    for path in sorted(RUST_ROOT.rglob("*.rs")):
        if is_platform_file(path) or path in ALLOWED_NON_PLATFORM_FILES:
            continue
        text = path.read_text(encoding="utf-8")
        for line_number, line in enumerate(text.splitlines(), start=1):
            for pattern, explanation in FORBIDDEN_PORTABLE_PATTERNS.items():
                if pattern in line:
                    problems.append(
                        f"{path}:{line_number}: {explanation}: found {pattern!r}"
                    )

    if problems:
        print("Platform-boundary guard failed:")
        print("\n".join(problems))
        return 1

    print("Platform-boundary guard passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
