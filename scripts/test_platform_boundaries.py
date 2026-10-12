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
            "use crate::platform::{windows::WindowsPlatform};\n",
            "use crate::platform::{linux::LinuxPlatform};\n",
            "use windows_sys as winapi;\n",
            "use windows_sys::{Win32::Foundation};\n",
            "use windows::Win32::Foundation;\n",
        ):
            with self.subTest(source=source):
                status, report = self.run_check({"app.rs": source})
                self.assertEqual(status, 1, report)

    def test_whitespace_variants_and_session_environment_are_rejected(self) -> None:
        for source in (
            'let _ = std::env::var ( "DISPLAY" );\n',
            'let _ = std::env::var_os ( "WAYLAND_DISPLAY" );\n',
            'let _ = std::env::var ( "XDG_CURRENT_DESKTOP" );\n',
            'let _ = std::env::var("DESKTOP_SESSION");\n',
            'let _ = std::env::var_os("GDMSESSION");\n',
        ):
            with self.subTest(source=source):
                status, report = self.run_check({"app.rs": source})
                self.assertEqual(status, 1, report)

    def test_multiline_imports_and_environment_probes_are_rejected(self) -> None:
        for source in (
            "use crate::platform\n  ::windows::capture;\n",
            "use crate::platform::{\n windows::WindowsPlatform\n};\n",
            "use windows_sys\n  ::Win32::Foundation;\n",
            'let _ = std::env::var(\n "DISPLAY"\n);\n',
            'let _ = std::env::var_os(\n "XDG_CURRENT_DESKTOP"\n);\n',
        ):
            with self.subTest(source=source):
                status, report = self.run_check({"app.rs": source})
                self.assertEqual(status, 1, report)
                self.assertIn("app.rs:1:", report)

    def test_platform_specific_os_apis_are_rejected_from_portable_code(self) -> None:
        for source in (
            "use std::os::windows::ffi::OsStrExt;\n",
            "use std::os::linux::raw::stat;\n",
            "use winapi::um::winuser;\n",
            "use winapi::shared::windef;\n",
            "use std::os\n  ::windows::ffi::OsStrExt;\n",
            "use winapi\n  ::um::winuser;\n",
        ):
            with self.subTest(source=source):
                status, report = self.run_check({"app.rs": source})
                self.assertEqual(status, 1, report)
                self.assertIn("app.rs:1:", report)

    def test_windows_registry_and_profile_paths_stay_outside_core(self) -> None:
        for source in (
            'let _ = std::env::var("APPDATA");\n',
            'let _ = std::env::var_os("LOCALAPPDATA");\n',
            'let _ = std::env::var("userprofile");\n',
            'let _ = std::env::var_os("PROGRAMFILES");\n',
            'use winreg::RegKey;\n',
            'use windows_registry as registry;\n',
            'let _ = HKEY_CURRENT_USER;\n',
            'let _ = std::env::var_os(\n "SYSTEMROOT"\n);\n',
        ):
            with self.subTest(source=source):
                status, report = self.run_check({"app.rs": source})
                self.assertEqual(status, 1, report)
                self.assertIn("app.rs:1:", report)

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

    def test_settings_file_permissions_must_use_platform_contract(self) -> None:
        for source in (
            "use std::os::unix::fs::PermissionsExt;\n",
            "fs::set_permissions(path, permissions)?;\n",
            "#[cfg(unix)]\nfn secure_settings() {}\n",
            "#[cfg(not(unix))]\nfn secure_settings() {}\n",
            '#[cfg(target_os = "windows")]\nfn secure_settings() {}\n',
        ):
            with self.subTest(source=source):
                status, report = self.run_check({"settings.rs": source})
                self.assertEqual(status, 1, report)
                self.assertIn("OS-specific settings file policy", report)
        status, report = self.run_check({
            "settings.rs": (
                "fn portable_settings() {}\n"
                "#[cfg(test)]\nmod tests {\n"
                "    use std::os::unix::fs::PermissionsExt;\n}\n"
            )
        })
        self.assertEqual(status, 0, report)

    def test_error_contract_rejects_target_gating_but_allows_test_cfg(self) -> None:
        for source in (
            '#[cfg(target_os = "linux")]\nenum ErrorCode {}\n',
            '#[cfg(not(any(target_os = "linux", target_os = "windows")))]\nenum AppError {}\n',
            '#[cfg_attr(target_os = "windows", allow(dead_code))]\nenum AppError {}\n',
        ):
            with self.subTest(source=source):
                status, report = self.run_check({"error.rs": source})
                self.assertEqual(status, 1, report)
                self.assertIn("platform-gated public error contract", report)
        status, report = self.run_check(
            {"error.rs": "#[cfg(test)]\nmod tests {}\n"}
        )
        self.assertEqual(status, 0, report)

    def test_linked_rust_source_is_rejected_without_reading_link_target(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "src-tauri" / "src"
            root.mkdir(parents=True)
            source = root / "ordinary.rs"
            source.write_text("fn ordinary() {}\\n", encoding="utf-8")
            (root / "linked.rs").symlink_to(source)
            output = io.StringIO()
            with (
                patch.object(checker, "RUST_ROOT", root),
                patch.object(checker, "PLATFORM_ROOT", root / "platform"),
                contextlib.redirect_stdout(output),
            ):
                status = checker.main()
            self.assertEqual(status, 1)
            self.assertIn("linked.rs: symlinked Rust source path is forbidden", output.getvalue())

    def test_linked_platform_directory_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "src-tauri" / "src"
            root.mkdir(parents=True)
            target = root / "ordinary"
            target.mkdir()
            (target / "mod.rs").write_text("fn ordinary() {}\\n", encoding="utf-8")
            (root / "platform").symlink_to(target, target_is_directory=True)
            output = io.StringIO()
            with (
                patch.object(checker, "RUST_ROOT", root),
                patch.object(checker, "PLATFORM_ROOT", root / "platform"),
                contextlib.redirect_stdout(output),
            ):
                status = checker.main()
            self.assertEqual(status, 1)
            self.assertIn("platform: symlinked Rust source path is forbidden", output.getvalue())

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
