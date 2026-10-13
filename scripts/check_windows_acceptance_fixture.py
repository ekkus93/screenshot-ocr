#!/usr/bin/env python3
"""Fail closed if the public Windows OCR desktop fixture gains external resources.

This is a deterministic source-content check, NOT physical Windows acceptance.
No screenshot, clipboard content, machine details, or OCR output are collected.
"""
from __future__ import annotations

import argparse
from collections import Counter
from html.parser import HTMLParser
from pathlib import Path
import re

REQUIRED_IDS = (
    "LIGHT-01", "DARK-02", "MONO-03", "SIZE-04",
    "PUNCT-05", "EDGE-06", "GRID-07",
)
ALLOWED_ATTRIBUTES = {
    "html": {"lang"},
    "head": set(),
    "meta": {"charset", "name", "content"},
    "title": set(),
    "style": set(),
    "body": set(),
    "h1": set(),
    "h2": set(),
    "p": {"class"},
    "section": {"class", "data-fixture-id"},
    "div": {"class"},
    "span": {"class"},
    "pre": {"class"},
}
EXPECTED_MONO = (
    'SAMPLE 123\n$ echo "synthetic text"\n'
    "const value = [1, 2, 3];\n"
    "  indented    spaces    stay visible"
)


class FixtureParser(HTMLParser):
    def __init__(self) -> None:
        super().__init__(convert_charrefs=True)
        self.errors: list[str] = []
        self.ids: list[str] = []
        self.style = False
        self.pre = False
        self.mono: list[str] = []

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        allowed = ALLOWED_ATTRIBUTES.get(tag)
        if allowed is None:
            self.errors.append("Forbidden HTML element in offline fixture.")
            return
        for name, _value in attrs:
            if name not in allowed:
                self.errors.append("Forbidden HTML attribute in offline fixture.")
        if tag == "section":
            self.ids.extend(
                value for name, value in attrs
                if name == "data-fixture-id" and value is not None
            )
        if tag == "style":
            self.style = True
        if tag == "pre":
            self.pre = True

    def handle_startendtag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        self.handle_starttag(tag, attrs)

    def handle_endtag(self, tag: str) -> None:
        if tag == "style":
            self.style = False
        if tag == "pre":
            self.pre = False

    def handle_data(self, data: str) -> None:
        if self.style and re.search(r"@import|url\s*\(|expression\s*\(", data, re.I):
            self.errors.append("External or executable CSS is forbidden.")
        if self.pre:
            self.mono.append(data)


def validate(content: str) -> list[str]:
    parser = FixtureParser()
    parser.feed(content)
    parser.close()
    errors = list(parser.errors)
    counts = Counter(parser.ids)
    if counts != Counter(REQUIRED_IDS):
        errors.append("Required public fixture IDs must occur exactly once.")
    if "".join(parser.mono) != EXPECTED_MONO:
        errors.append("Monospace OCR fixture whitespace changed.")
    if "<!doctype html>" not in content.lower():
        errors.append("Offline fixture must declare HTML doctype.")
    return errors


def main() -> int:
    arg = argparse.ArgumentParser(description=__doc__)
    arg.add_argument("fixture", type=Path)
    path = arg.parse_args().fixture
    if not path.is_file() or path.is_symlink():
        arg.exit(1, "Offline Windows acceptance fixture missing or redirected.\n")
    try:
        errors = validate(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, ValueError):
        arg.exit(1, "Offline Windows acceptance fixture unreadable.\n")
    if errors:
        arg.exit(1, "\n".join(errors) + "\n")
    print("Public offline Windows OCR fixture: static, complete and whitespace-stable.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
