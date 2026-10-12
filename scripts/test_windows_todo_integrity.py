"""Synthetic negative tests for Windows TODO truncation/integration guard."""
from __future__ import annotations

import importlib.util
import unittest
from pathlib import Path

MODULE_PATH = Path(__file__).with_name("check_windows_todo_integrity.py")
SPEC = importlib.util.spec_from_file_location("windows_todo_integrity", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
guard = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(guard)


def fixture() -> str:
    parts = [
        "# Screenshot OCR Windows Portability and Windows Application TODO",
        "# PHASE 1 — PORTABLE CORE REFACTOR",
        "# PHASE 2 — WINDOWS APPLICATION",
    ]
    for i in range(26):
        parts.extend([f"## {i + 4}. W{i} — test milestone", "- [ ] synthetic fixture"])
    parts.extend(guard.REQUIRED_MARKERS[3:])
    return "\n".join(parts) + "\n"


class WindowsTodoIntegrityTests(unittest.TestCase):
    def test_accepts_complete_structure(self) -> None:
        self.assertEqual(guard.validate(fixture(), minimum_checkboxes=26), [])

    def test_rejects_truncated_tail(self) -> None:
        damaged = fixture().split("## 28. W24")[0]
        self.assertTrue(any("W24" in problem for problem in guard.validate(damaged, 26)))

    def test_rejects_removed_physical_and_merge_gate(self) -> None:
        damaged = fixture().replace("- [ ] Required Windows physical acceptance passed.\n", "")
        damaged = damaged.replace("- [ ] Do not merge to master automatically.\n", "")
        problems = guard.validate(damaged, 26)
        self.assertTrue(any("physical acceptance" in problem for problem in problems))
        self.assertTrue(any("Do not merge" in problem for problem in problems))

    def test_rejects_duplicate_and_out_of_order_milestone(self) -> None:
        original = fixture()
        duplicate = original + "## 10. W6 — duplicate\n"
        self.assertTrue(any("W6" in e for e in guard.validate(duplicate, 26)))
        swapped = original.replace(
            "## 9. W5 — test milestone", "## 10. W6 — test milestone", 1
        ).replace(
            "## 10. W6 — test milestone\n- [ ] synthetic fixture\n"
            "## 10. W6 — test milestone",
            "## 9. W5 — test milestone\n- [ ] synthetic fixture\n"
            "## 10. W6 — test milestone",
            1,
        )
        self.assertTrue(guard.validate(swapped, 26))

    def test_rejects_empty_sections_and_missing_newline(self) -> None:
        original = fixture()
        self.assertTrue(any("newline" in e for e in guard.validate(original.rstrip("\n"), 26)))
        empty = original.replace(
            "## 7. W3 — test milestone\n- [ ] synthetic fixture",
            "## 7. W3 — test milestone",
        )
        self.assertTrue(any("no checklist" in e for e in guard.validate(empty, 26)))


if __name__ == "__main__":
    unittest.main()
