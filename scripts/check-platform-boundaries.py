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
    (r"\b(?:winreg|windows_registry)\b",
     "portable code references Windows registry bindings"),
    (r"\bHKEY_(?:CURRENT_USER|LOCAL_MACHINE|CLASSES_ROOT|USERS|CURRENT_CONFIG)\b",
     "portable code references Windows registry roots"),
    (r'(?i)\bstd::env::(?:var|var_os)\s*\(\s*"(?:APPDATA|LOCALAPPDATA|USERPROFILE|PROGRAMFILES|SYSTEMROOT|WINDIR)"',
     "portable code reads a Windows-specific environment path"),
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
    ),    (
        r'(?i)\bstd\s*::\s*env\s*::\s*(?:var|var_os)\s*\(\s*"(?:APPDATA|LOCALAPPDATA|USERPROFILE|PROGRAMFILES|SYSTEMROOT|WINDIR)"',
        "portable code probes Windows-specific environment paths",
    ),
)

ALLOWED_NON_PLATFORM_FILES = {
    # The composition root is intentionally the single place that selects the
    # current target-specific platform service.
    RUST_ROOT / "platform.rs",
}


# Platform conditionals inside the shared production core are architectural
# leakage even when they do not import a native API. Inspect each balanced
# cfg/cfg_attr/cfg! expression so nested all()/any()/not() forms are covered.
PLATFORM_CFG_START = re.compile(r"(?:#\s*\[\s*cfg(?:_attr)?\s*\(|\bcfg\s*!\s*\()")
PLATFORM_CFG_TOKEN = re.compile(
    r"\b(?:target_os|target_family|target_arch|target_env|target_vendor|windows|unix|linux)\b"
)


def production_platform_cfg_lines(source: str) -> list[int]:
    locations = []
    for match in PLATFORM_CFG_START.finditer(source):
        # Ignore source examples in full-line Rust comments.
        line_start = source.rfind("\n", 0, match.start()) + 1
        if source[line_start:match.start()].lstrip().startswith("//"):
            continue
        depth = 1
        offset = match.end()
        while offset < len(source) and depth:
            if source[offset] == "(":
                depth += 1
            elif source[offset] == ")":
                depth -= 1
            offset += 1
        if depth == 0 and PLATFORM_CFG_TOKEN.search(source[match.end():offset - 1]):
            locations.append(source.count("\n", 0, match.start()) + 1)
    return locations


def is_platform_file(path: Path) -> bool:
    return PLATFORM_ROOT in path.parents or path == PLATFORM_ROOT


def main() -> int:
    problems: list[str] = []
    for path in sorted(RUST_ROOT.rglob("*")):
        if path.is_symlink() and (path.suffix == ".rs" or path.is_dir()):
            problems.append(f"{path}: symlinked Rust source path is forbidden")
    for path in sorted(RUST_ROOT.rglob("*.rs")):
        if path.is_symlink():
            continue
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

        # Ignore cfg use in the ordinary in-file unit-test module and in
        # stand-alone test fixtures, but reject platform selection in the
        # portable production core. Bootstrap files own only composition.
        if path.name not in {"lib.rs", "main.rs"} and not path.name.endswith("_tests.rs"):
            production = text.split("#[cfg(test)]\nmod tests", 1)[0]
            for line_number in production_platform_cfg_lines(production):
                problems.append(
                    f"{path}:{line_number}: platform-specific cfg outside adapter"
                )

        # The settings schema and persistence policy are portable. Unix chmod
        # and target-specific filesystem mechanics belong to platform adapters;
        # unit-test fixtures after the test module boundary are exempt.
        if path == RUST_ROOT / "settings.rs":
            production = text.split("#[cfg(test)]\nmod tests", 1)[0]
            if re.search(
                r"\bstd\s*::\s*os\s*::\s*(?:unix|windows|linux)\b"
                r"|\bfs\s*::\s*set_permissions\s*\("
                r"|#\s*\[\s*cfg\s*\(\s*(?:not\s*\(\s*)?(?:unix|windows|linux|target_os)\b",
                production,
            ):
                problems.append(f"{path}: OS-specific settings file policy outside platform adapter")

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
