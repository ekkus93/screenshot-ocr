#!/usr/bin/env python3
"""Fail closed on obvious first-party OCR-content logging surfaces.

This is a conservative source guard, not a full information-flow proof.
Report source locations and rule names only; never echo matching source lines.
"""

from __future__ import annotations

import re
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

RUST_OUTPUT = re.compile(r"\b(?:dbg|print|println|eprint|eprintln)\s*!\s*\(")
# Also reject macros imported directly with `use tracing::info` or
# `use log::warn`, not only their qualified invocation forms.
RUST_LOG_OUTPUT = re.compile(
    r"\b(?:(?:tracing|log)\s*::\s*)?(?:trace|debug|info|warn|error|event|log|span)\s*!\s*\("
)
FRONTEND_OUTPUT = re.compile(
    r"\bconsole\s*(?:\?\.|\.)\s*(?:log|debug|info|warn|error)\s*(?:\?\.)?\s*\("
)
FRONTEND_BRACKET_OUTPUT = re.compile(
    r"""\bconsole\s*(?:\?\.)?\s*\[\s*['"](?:log|debug|info|warn|error)['"]\s*\]\s*(?:\?\.)?\s*\("""
)
FORBIDDEN_RULES = {
    ".rs": (
        ("Rust debug/print macro", RUST_OUTPUT),
        ("Rust structured logging macro", RUST_LOG_OUTPUT),
    ),
    **{
        extension: (
            ("frontend console call", FRONTEND_OUTPUT),
            ("frontend bracket console call", FRONTEND_BRACKET_OUTPUT),
        )
        for extension in (".ts", ".tsx", ".js", ".jsx")
    },
}


def should_scan(path: Path) -> bool:
    return (
        path.is_file()
        and path.suffix in SOURCE_SUFFIXES
        and not any(part in IGNORED_PARTS for part in path.parts)
    )


def scan_tree(root: Path) -> list[str]:
    # Never traverse a redirected checkout root or any linked parent. This
    # is a privacy boundary, not merely a way to suppress false positives.
    absolute_root = root.absolute()
    if any(path.is_symlink() for path in (absolute_root, *absolute_root.parents)):
        return ["source scan root traverses a symbolic link"]
    problems: list[str] = []
    for path in sorted(root.rglob("*")):
        # Path.is_file() follows symlinks. Fail closed before any read,
        # including for linked source directories outside the checkout.
        if (
            not any(part in IGNORED_PARTS for part in path.parts)
            and path.is_symlink()
            and (path.suffix in SOURCE_SUFFIXES or path.is_dir())
        ):
            problems.append(f"{path}: symbolic link in source scan")
            continue
        if not should_scan(path):
            continue
        # Match against the whole file: source code can legally split macro
        # names, member access and argument lists across multiple lines.
        # Matching line-by-line silently misses those content-leakage paths.
        source = path.read_text(encoding="utf-8")
        for description, pattern in FORBIDDEN_RULES[path.suffix]:
            for match in pattern.finditer(source):
                line_number = source.count("\n", 0, match.start()) + 1
                # Never print raw matching source: it might include OCR text.
                problems.append(f"{path}:{line_number}: forbidden {description}")
    return problems


def main() -> int:
    problems = scan_tree(Path("."))
    if problems:
        print("Content-leakage source guard failed:")
        print("\n".join(problems))
        return 1
    print("Content-leakage source guard passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
