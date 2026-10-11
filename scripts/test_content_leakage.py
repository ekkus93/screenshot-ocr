#!/usr/bin/env python3
"""Synthetic secret and output-path regression tests for content leakage guard."""

from __future__ import annotations

import importlib.util
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("check-content-leakage.py")
SPEC = importlib.util.spec_from_file_location("check_content_leakage", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
guard = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(guard)


class ContentLeakageGuardTests(unittest.TestCase):
    def scan(self, files: dict[str, str]) -> list[str]:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for name, contents in files.items():
                path = root / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(contents, encoding="utf-8")
            return guard.scan_tree(root)

    def test_normal_application_code_is_permitted(self) -> None:
        self.assertEqual(
            self.scan({
                "src-tauri/src/app.rs": "fn empty() {}\n",
                "src/app.tsx": "const text = 'synthetic';\n",
            }),
            [],
        )

    def test_rust_output_macro_rejects_whitespace_variants(self) -> None:
        for name in ("dbg", "println", "eprintln"):
            with self.subTest(name=name):
                report = self.scan({
                    "src-tauri/src/app.rs": (
                        f"fn safe() {{}}\n{name} ! (\\"PRIVATE_SYNTHETIC_SENTINEL\\");\n"
                    )
                })
                self.assertEqual(len(report), 1, report)
                self.assertIn("app.rs:2", report[0])
                self.assertNotIn("PRIVATE_SYNTHETIC_SENTINEL", report[0])

    def test_frontend_console_forms_are_rejected(self) -> None:
        variants = [
            "console.log('PRIVATE_SYNTHETIC_SENTINEL');",
            "console . warn ( 'PRIVATE_SYNTHETIC_SENTINEL' );",
            "console?.error('PRIVATE_SYNTHETIC_SENTINEL');",
            "console ['debug'] ('PRIVATE_SYNTHETIC_SENTINEL');",
        ]
        for source in variants:
            with self.subTest(source=source):
                report = self.scan({"src/panel.tsx": source})
                self.assertEqual(len(report), 1, report)
                self.assertNotIn("PRIVATE_SYNTHETIC_SENTINEL", report[0])

    def test_ignored_generated_directories_do_not_trigger_guard(self) -> None:
        source = "console.log('PRIVATE_SYNTHETIC_SENTINEL');\n"
        self.assertEqual(
            self.scan({
                "node_modules/package/a.js": source,
                "dist/output.js": source,
                "target/snapshot.js": source,
                "coverage/result.ts": source,
                "src/app.py": source,
            }),
            [],
        )

    def test_multiple_findings_have_file_and_line_without_source_content(self) -> None:
        report = self.scan({
            "src/a.ts": "export const a = 1;\nconsole.error('PRIVATE_SYNTHETIC_SENTINEL');\n",
            "src-tauri/src/b.rs": "dbg!(\\"PRIVATE_SYNTHETIC_SENTINEL\\");\n",
        })
        self.assertEqual(len(report), 2)
        self.assertIn("a.ts:2", "\n".join(report))
        self.assertIn("b.rs:1", "\n".join(report))
        self.assertNotIn("PRIVATE_SYNTHETIC_SENTINEL", "\n".join(report))


if __name__ == "__main__":
    unittest.main()
