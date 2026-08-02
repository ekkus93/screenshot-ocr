#!/usr/bin/env python3
"""Fail closed on obvious first-party content-leaking debug surfaces.

This is intentionally conservative and source-focused. It does not prove that OCR
content can never leak, but it blocks accidental console logging, print macros,
and debug macros in production/test source before a richer fault-injection suite
exists.
"""

from __future__ import annotations

from pathlib import Path

SOURCE_SUFFIXES = {".rs", ".ts", ".tsx", ".js", ".jsx"}
IGNORED_PARTS = {
    ".git",
    "node_modules",
    "target",
    "dist",
    "coverage",
    "playwright-report",
}

FORBIDDEN_PATTERNS = {
    ".rs": ["dbg!(", "println!(", "eprintln!("],
    ".ts": ["console.log(", "console.debug(", "console.info(", "console.warn(", "console.error("],
    ".tsx": ["console.log(", "console.debug(", "console.info(", "console.warn(", "console.error("],
    ".js": ["console.log(", "console.debug(", "console.info(", "console.warn(", "console.error("],
    ".jsx": ["console.log(", "console.debug(", "console.info(", "console.warn(", "console.error("],
}


def should_scan(path: Path) -> bool:
    return (
        path.is_file()
        and path.suffix in SOURCE_SUFFIXES
        and not any(part in IGNORED_PARTS for part in path.parts)
    )


def main() -> int:
    problems: list[str] = []
    for path in Path(".").rglob("*"):
        if not should_scan(path):
            continue
        patterns = FORBIDDEN_PATTERNS.get(path.suffix, [])
        text = path.read_text(encoding="utf-8")
        for line_number, line in enumerate(text.splitlines(), start=1):
            for pattern in patterns:
                if pattern in line:
                    problems.append(f"{path}:{line_number}: forbidden debug/logging pattern {pattern!r}")

    if problems:
        print("Content-leakage source guard failed:")
        print("\n".join(problems))
        return 1

    print("Content-leakage source guard passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
