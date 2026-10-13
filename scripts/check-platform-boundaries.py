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
    (r"\b(?:crate::)?platform::\{[^}]*\b(?:linux|windows)\b",
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
        r"\b(?:crate\s*::\s*)?platform\s*::\s*\{[^}]*\b(?:linux|windows)\b",
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


# Direct Win32 loader and known-folder calls must stay inside platform adapters,
# even when the bindings are imported under an alias or not spelled in a use line.
NATIVE_WINDOWS_MECHANISM_CALL = re.compile(
    r"\b(?:LoadLibrary(?:Ex)?[AW]?|SetDefaultDllDirectories|"
    r"AddDllDirectory|SetDllDirectory[AW]?|GetDllDirectory[AW]?|"
    r"SHGetKnownFolderPath|SHGetFolderPath[AW]?|"
    r"GetEnvironmentVariable[AW]?|GetTempPath[AW]?|"
    r"GetModuleFileName[AW]?)\s*\("
)


# Only an actual inline unit-test module ends portable production source.
# A marker embedded in a comment or raw string must not hide later code.
TEST_MODULE_START = re.compile(
    r"(?m)^[ \t]*#\[cfg\(test\)\][ \t]*\r?\n[ \t]*mod[ \t]+tests[ \t]*\{"
)


RAW_STRING_START = re.compile(r'(?:br|r)(#{0,255})"')


def is_rust_code_at(source: str, offset: int) -> bool:
    """Reject a test-module marker inside Rust comments or string literals.

    This conservative lexical preflight does not parse Rust syntax. It only
    decides whether the candidate marker starts outside a comment/string.
    """
    index = 0
    while index < offset:
        if source.startswith("//", index):
            end = source.find("\n", index)
            # A candidate before the comment's newline remains comment text:
            # skipping to offset made it incorrectly appear to be Rust code.
            if end < 0 or end >= offset:
                return False
            index = end + 1
        elif source.startswith("/*", index):
            depth = 1
            index += 2
            while depth and index < offset:
                if source.startswith("/*", index):
                    depth += 1
                    index += 2
                elif source.startswith("*/", index):
                    depth -= 1
                    index += 2
                else:
                    index += 1
            if depth:
                return False
        else:
            raw = (
                RAW_STRING_START.match(source, index)
                if index == 0 or not (source[index - 1].isalnum() or source[index - 1] == "_")
                else None
            )
            if raw:
                end_marker = '"' + raw.group(1)
                end = source.find(end_marker, raw.end())
                if end < 0 or end >= offset:
                    return False
                index = end + len(end_marker)
            elif source[index] == '"':
                index += 1
                while index < offset:
                    if source[index] == "\\":
                        index += 2
                    elif source[index] == '"':
                        index += 1
                        break
                    else:
                        index += 1
                else:
                    return False
            else:
                index += 1
    return True


def production_source(source: str) -> str:
    # Mask only the inline test module, not production that follows it.
    # Preserve newlines so diagnostic line numbers remain accurate.
    production = list(source)
    for match in TEST_MODULE_START.finditer(source):
        # Indentation is permitted before a real attribute. Compare the
        # entire candidate marker with the progressively masked source:
        # previously masked nested test modules are skipped, but whitespace
        # preceding a live marker must not make it look already masked.
        if (
            production[match.start():match.end()]
            != list(source[match.start():match.end()])
            or not is_rust_code_at(source, match.start())
        ):
            continue
        depth = 1
        opening = match.end() - 1
        for offset in range(opening + 1, len(source)):
            if source[offset] not in "{}" or not is_rust_code_at(source, offset):
                continue
            depth += 1 if source[offset] == "{" else -1
            if depth == 0:
                for index in range(match.start(), offset + 1):
                    if production[index] != "\n":
                        production[index] = " "
                break
    return "".join(production)


def production_platform_cfg_lines(source: str) -> list[int]:
    locations = []
    for match in PLATFORM_CFG_START.finditer(source):
        # A match inside a doc comment, nested block comment, or ordinary
        # / raw Rust string is an example, not an executable target gate.
        # Keep scanning after the example for real production cfg uses.
        if not is_rust_code_at(source, match.start()):
            continue
        depth = 1
        offset = match.end()
        while offset < len(source) and depth:
            # Delimiters in a feature value are not Rust cfg parentheses.
            if source[offset] in "()" and is_rust_code_at(source, offset):
                depth += 1 if source[offset] == "(" else -1
            offset += 1
        if depth != 0:
            continue
        # A quoted feature value named "windows" is not target selection.
        if any(
            is_rust_code_at(source, match.end() + token.start())
            for token in PLATFORM_CFG_TOKEN.finditer(source[match.end():offset - 1])
        ):
            locations.append(source.count("\n", 0, match.start()) + 1)
    return locations


def is_platform_file(path: Path) -> bool:
    return PLATFORM_ROOT in path.parents or path == PLATFORM_ROOT


def main() -> int:
    # A redirected checkout source root can make an otherwise read-only
    # boundary audit inspect files outside the repository. Reject the path
    # before traversing or reading any source files.
    absolute_root = RUST_ROOT.absolute()
    if any(path.is_symlink() for path in (absolute_root, *absolute_root.parents)):
        print("Platform-boundary guard failed: Rust source root traverses a symlink")
        return 1
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
            production = production_source(text)
            for line_number in production_platform_cfg_lines(production):
                problems.append(
                    f"{path}:{line_number}: platform-specific cfg outside adapter"
                )

        # These concrete Windows primitives have no place in portable
        # production modules. Ignore comments, string fixtures and unit tests;
        # preserve the diagnostic's repository-relative line number.
        if path.name not in {"lib.rs", "main.rs"} and not path.name.endswith("_tests.rs"):
            production = production_source(text)
            for match in NATIVE_WINDOWS_MECHANISM_CALL.finditer(production):
                if is_rust_code_at(production, match.start()):
                    line_number = production.count("\n", 0, match.start()) + 1
                    problems.append(
                        f"{path}:{line_number}: native Windows mechanism outside platform adapter"
                    )

        # Unix-only std::os extensions are implementation details of the
        # Linux adapter, not portable production code. Test fixtures may use
        # these extensions to validate platform behavior without leaking them
        # into the shipping core.
        if path.name not in {"lib.rs", "main.rs"} and not path.name.endswith("_tests.rs"):
            production = production_source(text)
            for match in re.finditer(r"\bstd\s*::\s*os\s*::\s*unix\b", production):
                line_number = production.count("\n", 0, match.start()) + 1
                problems.append(
                    f"{path}:{line_number}: portable code imports Unix-only OS APIs"
                )

        # The settings schema and persistence policy are portable. Unix chmod
        # and target-specific filesystem mechanics belong to platform adapters;
        # unit-test fixtures after the test module boundary are exempt.
        if path == RUST_ROOT / "settings.rs":
            production = production_source(text)
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
