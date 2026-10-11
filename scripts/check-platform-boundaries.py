#!/usr/bin/env python3
"""Enforce the portable-core/platform-adapter dependency boundary."""

from __future__ import annotations

import re
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

# These catch valid Rust import/probe spellings that do not contain the
# literal substrings above (grouped imports, crate aliases, whitespace).
FORBIDDEN_PORTABLE_REGEX = (
    (r"\b(?:crate::)?platform::\{\s*(?:linux|windows)\b",
     "portable code imports a concrete adapter through a grouped path"),
    (r"\bwindows_sys\b(?!::Win32)",
     "portable code imports or aliases the native Windows API crate"),
    (r"\bwindows::Win32\b",
     "portable code imports a concrete Windows API"),
    (r"\b(?:winapi\s*::\s*(?:um|shared)|std\s*::\s*os\s*::\s*(?:windows|linux))\b",
     "portable code imports platform-specific OS APIs"),
    (r'\bstd::env::(?:var|var_os)\s*\(\s*"(?:DESKTOP_SESSION|GDMSESSION)"',
     "portable code reads a Linux desktop session environment variable"),
    (r'\bstd::env::(?:var|var_os)\s+\(\s*"(?:DISPLAY|WAYLAND_DISPLAY|XDG_[A-Za-z0-9_]*|GNOME_[A-Za-z0-9_]*)"',
     "portable code reads a Linux desktop environment variable"),
)

# Check complete source text as well: Rust permits line breaks between path
# segments and function arguments, which line-by-line guards cannot see.
FORBIDDEN_PORTABLE_MULTILINE_REGEX = (
    (
        r"\b(?:crate\s*::\s*)?platform\s*::\s*(?:linux|windows)\b",
        "portable code imports a concrete platform adapter",
    ),
    (
        r"\b(?:crate\s*::\s*)?platform\s*::\s*\{\s*(?:linux|windows)\b",
        "portable code imports a grouped platform adapter",
    ),
    (
        r"\b(?:windows_sys|windows)\s*::\s*Win32\b",
        "portable code imports native Windows APIs",
    ),
    (
        r"\b(?:winapi\s*::\s*(?:um|shared)|std\s*::\s*os\s*::\s*(?:windows|linux))\b",
        "portable code imports platform-specific OS APIs",
    ),
    (
        r'\bstd\s*::\s*env\s*::\s*(?:var|var_os)\s*\(\s*"(?:DISPLAY|WAYLAND_DISPLAY|XDG_[A-Za-z0-9_]*|GNOME_[A-Za-z0-9_]*|DESKTOP_SESSION|GDMSESSION)"',
        "portable code probes Linux desktop environment",
    ),
)

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
            for expression, explanation in FORBIDDEN_PORTABLE_REGEX:
                if re.search(expression, line):
                    problems.append(
                        f"{path}:{line_number}: {explanation}: matched {expression!r}"
                    )

        # Error variants/codes must be available identically on every target.
        if path == RUST_ROOT / "error.rs" and re.search(
            r"#\s*\[\s*cfg(?:_attr)?\s*\([^\]]*target_os", text
        ):
            problems.append(f"{path}: platform-gated public error contract")

        for expression, explanation in FORBIDDEN_PORTABLE_MULTILINE_REGEX:
            for match in re.finditer(expression, text):
                # Single-line matches are already handled above.
                if "\n" not in match.group(0):
                    continue
                line_number = text.count("\n", 0, match.start()) + 1
                problems.append(
                    f"{path}:{line_number}: {explanation}: matched {expression!r}"
                )

    if problems:
        print("Platform-boundary guard failed:")
        print("\n".join(problems))
        return 1

    print("Platform-boundary guard passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
