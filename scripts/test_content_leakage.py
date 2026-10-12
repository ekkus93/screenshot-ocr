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
        for name in ("dbg", "print", "println", "eprint", "eprintln"):
            with self.subTest(name=name):
                report = self.scan({
                    "src-tauri/src/app.rs": (
                        f'fn safe() {{}}\n{name} ! ("PRIVATE_SYNTHETIC_SENTINEL");\n'
                    )
                })
                self.assertEqual(len(report), 1, report)
                self.assertIn("app.rs:2", report[0])
                self.assertNotIn("PRIVATE_SYNTHETIC_SENTINEL", report[0])

    def test_rust_structured_logging_macros_are_rejected_without_source_text(self) -> None:
        for source in (
            'tracing::info!(message = "PRIVATE_SYNTHETIC_SENTINEL");',
            'tracing :: event ! (tracing::Level::INFO, "PRIVATE_SYNTHETIC_SENTINEL");',
            'log::warn!("PRIVATE_SYNTHETIC_SENTINEL");',
        ):
            with self.subTest(source=source):
                report = self.scan({"src-tauri/src/app.rs": source})
                self.assertEqual(len(report), 1, report)
                self.assertIn("Rust structured logging macro", report[0])
                self.assertNotIn("PRIVATE_SYNTHETIC_SENTINEL", report[0])

    def test_imported_rust_logging_macros_are_rejected(self) -> None:
        for name in ("trace", "debug", "info", "warn", "error", "event", "span"):
            with self.subTest(name=name):
                report = self.scan({
                    "src-tauri/src/app.rs": (
                        f'use tracing::{name};\n{name}!("PRIVATE_SYNTHETIC_SENTINEL");\n'
                    )
                })
                self.assertEqual(len(report), 1, report)
                self.assertIn("Rust structured logging macro", report[0])
                self.assertNotIn("PRIVATE_SYNTHETIC_SENTINEL", report[0])

    def test_frontend_console_forms_are_rejected(self) -> None:
        variants = [
            "console.log('PRIVATE_SYNTHETIC_SENTINEL');",
            "console . warn ( 'PRIVATE_SYNTHETIC_SENTINEL' );",
            "console?.error('PRIVATE_SYNTHETIC_SENTINEL');",
            "console ['debug'] ('PRIVATE_SYNTHETIC_SENTINEL');",
            "console?.['warn']('PRIVATE_SYNTHETIC_SENTINEL');",
            "console?.['warn']?.('PRIVATE_SYNTHETIC_SENTINEL');",
            "console.log?.('PRIVATE_SYNTHETIC_SENTINEL');",
            "console?.error?.('PRIVATE_SYNTHETIC_SENTINEL');",
        ]
        for source in variants:
            with self.subTest(source=source):
                report = self.scan({"src/panel.tsx": source})
                self.assertEqual(len(report), 1, report)
                self.assertNotIn("PRIVATE_SYNTHETIC_SENTINEL", report[0])

    def test_extended_console_diagnostics_are_rejected_without_source_text(self) -> None:
        methods = (
            "trace", "table", "dir", "dirxml", "assert", "count",
            "countReset", "group", "groupCollapsed", "groupEnd",
            "time", "timeEnd", "timeLog",
        )
        for method in methods:
            for expression in (
                f"console.{method}('PRIVATE_SYNTHETIC_SENTINEL');",
                f"console?.['{method}']?.('PRIVATE_SYNTHETIC_SENTINEL');",
            ):
                with self.subTest(method=method, expression=expression):
                    report = self.scan({"src/panel.tsx": expression})
                    self.assertEqual(len(report), 1, report)
                    self.assertNotIn("PRIVATE_SYNTHETIC_SENTINEL", report[0])

    def test_multiline_output_calls_cannot_bypass_guard(self) -> None:
        fixtures = (
            ("src-tauri/src/app.rs", 'println!\n ( "PRIVATE_SYNTHETIC_SENTINEL" );'),
            ("src-tauri/src/app.rs", 'tracing\n :: info!\n ("PRIVATE_SYNTHETIC_SENTINEL");'),
            ("src/panel.tsx", 'console\n . error\n ("PRIVATE_SYNTHETIC_SENTINEL");'),
            ("src/panel.tsx", 'console\n ["warn"]\n ("PRIVATE_SYNTHETIC_SENTINEL");'),
        )
        for filename, source in fixtures:
            with self.subTest(filename=filename, source=source):
                report = self.scan({filename: source})
                self.assertEqual(len(report), 1, report)
                self.assertIn(f"{Path(filename).name}:1", report[0])
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

    def test_symlinked_source_is_rejected_without_reading_private_target(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            directory = Path(tmp)
            checkout = directory / "checkout"
            checkout.mkdir()
            outside = directory / "private.rs"
            outside.write_text("PRIVATE_SYNTHETIC_SENTINEL", encoding="utf-8")
            link = checkout / "linked.rs"
            try:
                link.symlink_to(outside)
            except (OSError, NotImplementedError):
                self.skipTest("host does not permit source symlinks")
            report = guard.scan_tree(checkout)
            self.assertEqual(len(report), 1, report)
            self.assertIn("symbolic link in source scan", report[0])
            self.assertNotIn("PRIVATE_SYNTHETIC_SENTINEL", report[0])

    def test_symlinked_ancestor_root_fails_closed_without_reading_content(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            directory = Path(tmp)
            real = directory / "actual"
            checkout = real / "src"
            checkout.mkdir(parents=True)
            (checkout / "safe.rs").write_text("fn empty() {}", encoding="utf-8")
            alias = directory / "alias"
            try:
                alias.symlink_to(real, target_is_directory=True)
            except (OSError, NotImplementedError):
                self.skipTest("host does not permit directory symlinks")
            report = guard.scan_tree(alias / "src")
            self.assertEqual(report, ["source scan root traverses a symbolic link"])

    def test_relative_scan_root_detects_linked_parent(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            base = Path(tmp)
            actual = base / "actual" / "src"
            actual.mkdir(parents=True)
            (actual / "safe.rs").write_text("fn safe() {}", encoding="utf-8")
            linked = base / "linked"
            try:
                linked.symlink_to(base / "actual", target_is_directory=True)
            except (OSError, NotImplementedError):
                self.skipTest("host does not permit directory symlinks")
            old_cwd = Path.cwd()
            try:
                import os
                os.chdir(base)
                self.assertEqual(
                    guard.scan_tree(Path("linked") / "src"),
                    ["source scan root traverses a symbolic link"],
                )
            finally:
                os.chdir(old_cwd)

    def test_dangling_and_unknown_extension_links_cannot_hide_source(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp) / "checkout"
            root.mkdir()
            outside = Path(tmp) / "private.data"
            outside.write_text("PRIVATE_SYNTHETIC_SENTINEL", encoding="utf-8")
            link = root / "src" / "opaque.data"
            link.parent.mkdir(parents=True)
            try:
                link.symlink_to(outside)
                dangling = root / "src" / "missing-source-tree"
                dangling.symlink_to(
                    Path(tmp) / "not-present", target_is_directory=True
                )
            except (OSError, NotImplementedError):
                self.skipTest("host does not permit source symlinks")
            report = guard.scan_tree(root)
            self.assertEqual(
                report,
                [
                    "src/missing-source-tree: symbolic link in source scan",
                    "src/opaque.data: symbolic link in source scan",
                ],
            )
            self.assertNotIn("PRIVATE_SYNTHETIC_SENTINEL", str(report))

    def test_generated_symlink_is_not_treated_as_first_party_source(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp) / "checkout"
            root.mkdir()
            outside = Path(tmp) / "fixture.js"
            outside.write_text("console.log('PRIVATE_SYNTHETIC_SENTINEL')", encoding="utf-8")
            generated = root / "node_modules"
            generated.mkdir()
            try:
                (generated / "linked.js").symlink_to(outside)
            except (OSError, NotImplementedError):
                self.skipTest("host does not permit source symlinks")
            self.assertEqual(guard.scan_tree(root), [])

    def test_diagnostics_do_not_disclose_absolute_checkout_location(self) -> None:
        with tempfile.TemporaryDirectory(prefix="PRIVATE_RUNNER_PROFILE_") as tmp:
            root = Path(tmp) / "private" / "checkout"
            source = root / "src" / "panel.tsx"
            source.parent.mkdir(parents=True)
            source.write_text(
                "console.error('PRIVATE_SYNTHETIC_SENTINEL')\\n",
                encoding="utf-8",
            )
            problems = guard.scan_tree(root)
            self.assertEqual(
                problems, ["src/panel.tsx:1: forbidden frontend console call"]
            )
            self.assertNotIn(str(root), str(problems))
            self.assertNotIn("PRIVATE_RUNNER_PROFILE_", str(problems))

    def test_invalid_utf8_is_reported_without_absolute_paths_or_bytes(self) -> None:
        with tempfile.TemporaryDirectory(prefix="PRIVATE_RUNNER_PROFILE_") as tmp:
            root = Path(tmp) / "checkout"
            source = root / "src" / "malformed.ts"
            source.parent.mkdir(parents=True)
            source.write_bytes(bytes([0xFF]) + b"PRIVATE_SYNTHETIC_SENTINEL")
            problems = guard.scan_tree(root)
            self.assertEqual(
                problems, ["src/malformed.ts: unreadable first-party source"]
            )
            self.assertNotIn(str(root), str(problems))
            self.assertNotIn("PRIVATE_SYNTHETIC_SENTINEL", str(problems))

    def test_symlink_diagnostics_are_repository_relative(self) -> None:
        with tempfile.TemporaryDirectory(prefix="PRIVATE_RUNNER_PROFILE_") as tmp:
            root = Path(tmp) / "checkout"
            root.mkdir()
            outside = Path(tmp) / "private.rs"
            outside.write_text("PRIVATE_SYNTHETIC_SENTINEL", encoding="utf-8")
            link = root / "linked.rs"
            try:
                link.symlink_to(outside)
            except (OSError, NotImplementedError):
                self.skipTest("host does not permit source symlinks")
            self.assertEqual(
                guard.scan_tree(root), ["linked.rs: symbolic link in source scan"]
            )

    def test_multiple_findings_have_file_and_line_without_source_content(self) -> None:
        report = self.scan({
            "src/a.ts": "export const a = 1;\nconsole.error('PRIVATE_SYNTHETIC_SENTINEL');\n",
            "src-tauri/src/b.rs": 'dbg!("PRIVATE_SYNTHETIC_SENTINEL");\n',
        })
        self.assertEqual(len(report), 2)
        self.assertIn("a.ts:2", "\n".join(report))
        self.assertIn("b.rs:1", "\n".join(report))
        self.assertNotIn("PRIVATE_SYNTHETIC_SENTINEL", "\n".join(report))


if __name__ == "__main__":
    unittest.main()
