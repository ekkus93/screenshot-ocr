#!/usr/bin/env python3
"""Synthetic regression tests for the portable/platform source boundary guard."""

from __future__ import annotations

import contextlib
import importlib.util
import io
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch


CHECKER_PATH = Path(__file__).with_name("check-platform-boundaries.py")
SPEC = importlib.util.spec_from_file_location("check_platform_boundaries", CHECKER_PATH)
assert SPEC is not None and SPEC.loader is not None
checker = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(checker)


class PlatformBoundaryGuardTests(unittest.TestCase):
    def run_check(self, files: dict[str, str]) -> tuple[int, str]:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "src-tauri" / "src"
            root.mkdir(parents=True)
            for name, source in files.items():
                target = root / name
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_text(source, encoding="utf-8")
            output = io.StringIO()
            with (
                patch.object(checker, "RUST_ROOT", root),
                patch.object(checker, "PLATFORM_ROOT", root / "platform"),
                patch.object(
                    checker, "ALLOWED_NON_PLATFORM_FILES", {root / "platform.rs"}
                ),
                contextlib.redirect_stdout(output),
            ):
                status = checker.main()
            return status, output.getvalue()

    def test_portable_core_can_use_domain_types(self) -> None:
        status, report = self.run_check(
            {"app.rs": "use crate::models::CaptureResult;\n"}
        )
        self.assertEqual(status, 0, report)
        self.assertIn("passed", report)

    def test_concrete_linux_and_windows_imports_are_rejected(self) -> None:
        for source in (
            "use crate::platform::linux::capture;\n",
            "use crate::platform::windows::capture;\n",
            "use ashpd::desktop::screenshot;\n",
            "use windows_sys::Win32::Foundation;\n",
            "let _ = GnomeScreenshotBackend;\n",
            "let _ = PortalScreenshotBackend;\n",
        ):
            with self.subTest(source=source):
                status, report = self.run_check({"app.rs": source})
                self.assertEqual(status, 1, report)
                self.assertIn("app.rs:1:", report)

    def test_desktop_environment_probe_is_rejected_from_core(self) -> None:
        for name in ("DISPLAY", "WAYLAND_DISPLAY", "XDG_CURRENT_DESKTOP", "GNOME_DESKTOP_SESSION_ID"):
            for method in ("var", "var_os"):
                with self.subTest(name=name, method=method):
                    status, report = self.run_check(
                        {"app.rs": f'let _ = std::env::{method}("{name}");\n'}
                    )
                    self.assertEqual(status, 1, report)

    def test_grouped_imports_and_native_crate_aliases_are_rejected(self) -> None:
        for source in (
            "use crate::platform::{windows::WindowsPlatform};\\n",
            "use crate::platform::{linux::LinuxPlatform};\\n",
            "use windows_sys as winapi;\\n",
            "use windows_sys::{Win32::Foundation};\\n",
            "use windows::Win32::Foundation;\\n",
        ):
            with self.subTest(source=source):
                status, report = self.run_check({"app.rs": source})
                self.assertEqual(status, 1, report)

    def test_whitespace_variants_and_session_environment_are_rejected(self) -> None:
        for source in (
            'let _ = std::env::var ( "DISPLAY" );\\n',
            'let _ = std::env::var_os ( "WAYLAND_DISPLAY" );\\n',
            'let _ = std::env::var ( "XDG_CURRENT_DESKTOP" );\\n',
            'let _ = std::env::var("DESKTOP_SESSION");\\n',
            'let _ = std::env::var_os("GDMSESSION");\\n',
        ):
            with self.subTest(source=source):
                status, report = self.run_check({"app.rs": source})
                self.assertEqual(status, 1, report)

    def test_platform_adapter_can_use_native_imports(self) -> None:
        status, report = self.run_check(
            {
                "platform/linux/mod.rs": (
                    'use ashpd::desktop::screenshot;\n'
                    'let _ = std::env::var("XDG_CURRENT_DESKTOP");\n'
                ),
                "platform/windows/mod.rs": "use windows_sys::Win32::Foundation;\n",
            }
        )
        self.assertEqual(status, 0, report)

    def test_composition_root_is_the_only_non_adapter_exception(self) -> None:
        source = "use crate::platform::windows::WindowsPlatform;\n"
        status, report = self.run_check(
            {"platform.rs": source, "app.rs": "let result = true;\n"}
        )
        self.assertEqual(status, 0, report)
        status, report = self.run_check({"platform.rs": source, "app.rs": source})
        self.assertEqual(status, 1, report)
        self.assertIn("app.rs:1:", report)

    def test_reports_multiple_file_line_numbers(self) -> None:
        status, report = self.run_check(
            {
                "app.rs": 'fn safe() {}\nlet _ = std::env::var("DISPLAY");\n',
                "models.rs": "use crate::platform::windows::capture;\n",
            }
        )
        self.assertEqual(status, 1, report)
        self.assertIn("app.rs:2:", report)
        self.assertIn("models.rs:1:", report)


if __name__ == "__main__":
    unittest.main()
