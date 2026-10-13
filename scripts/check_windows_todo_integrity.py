#!/usr/bin/env python3
"""Fail closed if the authoritative Windows TODO loses milestone/acceptance gates.

This validates document structure, not whether any requirement is satisfied.
Keep physical and redistribution checkboxes open until they are proven.
"""
from __future__ import annotations

import argparse
from pathlib import Path

EXPECTED_MILESTONES = tuple(f"## {number + 4}. W{number} — " for number in range(26))
REQUIRED_MARKERS = (
    "# Screenshot OCR Windows Portability and Windows Application TODO",
    "# PHASE 1 — PORTABLE CORE REFACTOR",
    "# PHASE 2 — WINDOWS APPLICATION",
    "### W20.5 Artifact integrity",
    "### W21.4 Exact-head qualification",
    "### W23.1 Basic Windows 11 x86-64",
    "### W23.7 Windows 10 22H2",
    "### W25 final acceptance gate",
    "## 30. Explicitly deferred after this project",
    "- [ ] Required Windows physical acceptance passed.",
    "- [ ] Do not merge to master automatically.",
)


def validate(content: str, minimum_checkboxes: int = 100) -> list[str]:
    lines = content.splitlines()
    errors: list[str] = []
    positions: list[int] = []
    for heading in EXPECTED_MILESTONES:
        hits = [i for i, line in enumerate(lines) if line.startswith(heading)]
        if len(hits) != 1:
            errors.append(f"Expected exactly one milestone heading: {heading.strip()}")
        else:
            positions.append(hits[0])
    if len(positions) == len(EXPECTED_MILESTONES) and positions != sorted(positions):
        errors.append("Windows milestone headings are out of order.")
    for marker in REQUIRED_MARKERS:
        if marker not in lines:
            errors.append(f"Missing acceptance or integration marker: {marker}")
    for index, position in enumerate(positions):
        stop = positions[index + 1] if index + 1 < len(positions) else len(lines)
        if not any(line.startswith(("- [ ] ", "- [x] ")) for line in lines[position:stop]):
            errors.append(f"Milestone section at line {position + 1} has no checklist.")
    count = sum(line.startswith(("- [ ] ", "- [x] ")) for line in lines)
    if count < minimum_checkboxes:
        errors.append(f"Too few checklist items: {count} < {minimum_checkboxes}")
    if content and not content.endswith("\n"):
        errors.append("Authoritative TODO must end with a newline.")
    return errors


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("todo", type=Path)
    args = parser.parse_args()
    if not args.todo.is_file() or args.todo.is_symlink():
        parser.exit(1, "Authoritative Windows TODO is missing or redirected.\n")
    errors = validate(args.todo.read_text(encoding="utf-8"))
    if errors:
        parser.exit(1, "\n".join(errors) + "\n")
    print("Windows TODO milestones W0–W25, acceptance and integration gates intact.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
