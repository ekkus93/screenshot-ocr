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
            "use crate::platform::{self, windows::WindowsPlatform};\n",
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
            "use crate::platform::{\n self,\n linux::LinuxPlatform\n};\n",
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

    def test_unix_only_os_extensions_stay_outside_portable_production(self) -> None:
        for source in (
            "use std::os::unix::fs::PermissionsExt;\n",
            "use std::os\n  ::unix::ffi::OsStrExt;\n",
        ):
            with self.subTest(source=source):
                status, report = self.run_check({"app.rs": source})
                self.assertEqual(status, 1, report)
                self.assertIn("portable code imports Unix-only OS APIs", report)

        status, report = self.run_check({
            "app.rs": (
                "fn shared() {}\n#[cfg(test)]\nmod tests {\n"
                "  use std::os::unix::fs::PermissionsExt;\n}\n"
            ),
        })
        self.assertEqual(status, 0, report)

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

    def test_native_windows_loader_and_path_calls_are_confined_to_adapters(self) -> None:
        for call in (
            "LoadLibraryExW(name, 0, flags);",
            "SetDefaultDllDirectories(flags);",
            "AddDllDirectory(path);",
            "SHGetKnownFolderPath(id, 0, 0, out);",
            "SHGetFolderPathW(hwnd, id, 0, 0, out);",
            "GetEnvironmentVariableW(name, buffer, size);",
            "GetTempPathW(size, buffer);",
            "GetModuleFileNameW(handle, buffer, size);",
            "LoadLibraryExW\n  (name, 0, flags);",
        ):
            with self.subTest(call=call):
                status, report = self.run_check({"app.rs": call + "\n"})
                self.assertEqual(status, 1, report)
                self.assertIn("native Windows mechanism outside platform adapter", report)

        status, report = self.run_check({
            "app.rs": (
                'let child = std::process::Command::new("tesseract");\n'
                'const EXAMPLE: &str = r#"LoadLibraryExW(name, 0, 0);"#;\n'
                '// SHGetKnownFolderPath(id, 0, 0, out);\n'
                '#[cfg(test)]\nmod tests {\n'
                '  fn fixture() { LoadLibraryExW(name, 0, 0); }\n}\n'
            ),
            "platform/windows/mod.rs": "fn native() { LoadLibraryExW(name, 0, 0); }\n",
        })
        self.assertEqual(status, 0, report)

    def test_platform_cfg_stays_outside_portable_production(self) -> None:
        for source in (
            '#[cfg(target_os = "windows")]\nfn native() {}\n',
            '#[cfg(not(unix))]\nfn native() {}\n',
            '#[cfg(any(feature = "x", target_family = "windows"))]\nfn native() {}\n',
            '#[cfg_attr(target_arch = "x86_64", allow(dead_code))]\nfn native() {}\n',
            'let native = cfg!(target_os = "linux");\n',
            '#[cfg(all(\nfeature = "x",\nnot(target_os = "windows")\n))]\nfn native() {}\n',
        ):
            with self.subTest(source=source):
                status, report = self.run_check({"app.rs": source})
                self.assertEqual(status, 1, report)
                self.assertIn("platform-specific cfg outside adapter", report)

    def test_platform_cfg_in_test_only_modules_and_bootstrap_is_allowed(self) -> None:
        status, report = self.run_check({
            "app.rs": ('fn shared() {}\n#[cfg(test)]\nmod tests {\n'
                       '  #[cfg(unix)] fn only_unix_test() {}\n}\n'),
            "app_portability_tests.rs": '#[cfg(windows)]\nfn fixture() {}\n',
            "lib.rs": '#[cfg_attr(target_os = "windows", allow(dead_code))]\nfn entry() {}\n',
            "main.rs": '#[cfg(windows)]\nfn main() {}\n',
            "platform/windows/mod.rs": '#[cfg(target_os = "windows")]\nfn adapter() {}\n',
        })
        self.assertEqual(status, 0, report)

    def test_documentation_cannot_hide_production_cfg(self) -> None:
        # The old literal split treated this raw-string example as a real
        # unit-test boundary, silently skipping the production cfg below.
        status, report = self.run_check({
            "app.rs": (
                'const EXAMPLE: &str = r#"#[cfg(test)]\nmod tests {"#;\n'
                '#[cfg(windows)]\nfn native() {}\n'
            )
        })
        self.assertEqual(status, 1, report)
        self.assertIn("platform-specific cfg outside adapter", report)

        status, report = self.run_check({
            "app.rs": (
                "fn shared() {}\n  #[cfg(test)]  \n  mod tests {\n"
                "  #[cfg(windows)] fn test_only() {}\n}\n"
            )
        })
        self.assertEqual(status, 0, report)

    def test_indented_unit_test_module_remains_exempt_without_hiding_following_code(self) -> None:
        for indentation in (" ", "    ", "\t", " \t"):
            with self.subTest(indentation=repr(indentation)):
                code = (
                    "fn shared() {}\n"
                    + indentation + "#[cfg(test)]\n"
                    + indentation + "mod tests {\n"
                    + "    #[cfg(windows)] fn test_only() {}\n"
                    + "}\n"
                )
                status, report = self.run_check({"app.rs": code})
                self.assertEqual(status, 0, report)
                status, report = self.run_check({
                    "app.rs": code + '#[cfg(target_os = "windows")]\nfn production() {}\n'
                })
                self.assertEqual(status, 1, report)
                self.assertIn("platform-specific cfg outside adapter", report)

    def test_documented_platform_cfg_literals_are_not_production_attributes(self) -> None:
        examples = (
            'const TEXT: &str = r#"#[cfg(target_os = "windows")]"#;\n',
            'const TEXT: &str = r###"\n#[cfg(unix)]\n"###;\n',
            'const TEXT: &str = "#[cfg(windows)]";\n',
            '/* outer /* inner #[cfg(target_os = "linux")] */ outer */\n',
            '// #[cfg(windows)]\n',
        )
        for example in examples:
            with self.subTest(example=example):
                status, report = self.run_check({"app.rs": example + "fn portable() {}\n"})
                self.assertEqual(status, 0, report)
                source = example + '#[cfg(target_os = "windows")]\nfn native() {}\n'
                status, report = self.run_check({"app.rs": source})
                self.assertEqual(status, 1, report)
                self.assertIn("platform-specific cfg outside adapter", report)

    def test_cfg_string_values_do_not_masquerade_as_platform_gates(self) -> None:
        for source in (
            '#[cfg(feature = "windows")]\nfn portable() {}\n',
            '#[cfg(feature = "unix")]\nfn portable() {}\n',
            '#[cfg(feature = "target_os")]\nfn portable() {}\n',
            '#[cfg(all(feature = ")", feature = "linux"))]\nfn portable() {}\n',
        ):
            with self.subTest(source=source):
                status, report = self.run_check({"app.rs": source})
                self.assertEqual(status, 0, report)

    def test_parentheses_inside_cfg_strings_cannot_hide_actual_target_gate(self) -> None:
        source = '#[cfg(all(feature = ")", target_os = "windows"))]\nfn native() {}\n'
        status, report = self.run_check({"app.rs": source})
        self.assertEqual(status, 1, report)
        self.assertIn("platform-specific cfg outside adapter", report)
        source = '#[cfg_attr(feature = ")", target_os = "windows")]\nfn native() {}\n'
        status, report = self.run_check({"app.rs": source})
        self.assertEqual(status, 1, report)
        self.assertIn("platform-specific cfg outside adapter", report)

    def test_cfg_macro_inside_raw_string_does_not_trigger_platform_guard(self) -> None:
        code = 'const DOC: &str = r#"cfg!(target_os = "windows")"#;\n'
        status, report = self.run_check({"app.rs": code + "fn portable() {}\n"})
        self.assertEqual(status, 0, report)
        status, report = self.run_check({
            "app.rs": code + 'let native = cfg!(target_os = "windows");\n'
        })
        self.assertEqual(status, 1, report)

    def test_fake_test_module_in_multiline_literals_or_comments_does_not_hide_cfg(self) -> None:
        examples = (
            'const EXAMPLE: &str = r#"\n#[cfg(test)]\nmod tests {\n"#;\n',
            'const EXAMPLE: &str = r###"\n#[cfg(test)]\nmod tests {\n"###;\n',
            'const EXAMPLE: &str = "start\n#[cfg(test)]\nmod tests {\n";\n',
            '/*\n#[cfg(test)]\nmod tests {\n*/\n',
            '/* outer /* nested\n#[cfg(test)]\nmod tests {\n*/ */\n',
        )
        for prefix in examples:
            with self.subTest(prefix=prefix):
                status, report = self.run_check({
                    "app.rs": prefix + '#[cfg(target_os = "windows")]\nfn native() {}\n',
                })
                self.assertEqual(status, 1, report)
                self.assertIn("platform-specific cfg outside adapter", report)

    def test_real_test_module_after_fake_marker_is_still_exempt(self) -> None:
        status, report = self.run_check({
            "app.rs": (
                'const EXAMPLE: &str = r#"\n#[cfg(test)]\nmod tests {\n"#;\n'
                "fn portable() {}\n#[cfg(test)]\nmod tests {\n"
                "    #[cfg(windows)] fn native_test() {}\n}\n"
            ),
        })
        self.assertEqual(status, 0, report)

    def test_platform_cfg_after_test_module_is_still_production(self) -> None:
        status, report = self.run_check({
            "app.rs": (
                "fn shared() {}\n#[cfg(test)]\nmod tests {\n"
                "    fn nested() { let _ = (1, 2); }\n"
                "    #[cfg(windows)] fn test_only() {}\n}\n"
                '#[cfg(target_os = "windows")]\nfn native() {}\n'
            ),
        })
        self.assertEqual(status, 1, report)
        self.assertIn("app.rs:7:", report)
        self.assertIn("platform-specific cfg outside adapter", report)

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

    def test_linked_source_root_and_parent_fail_before_reading(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            actual = base / "actual"
            source = actual / "src-tauri" / "src"
            source.mkdir(parents=True)
            (source / "secret.rs").write_text(
                "GNOME PRIVATE_FIXTURE\\n", encoding="utf-8"
            )
            alias = base / "alias"
            alias.symlink_to(actual, target_is_directory=True)
            for linked_root in (alias / "src-tauri" / "src", base / "linked-src"):
                if linked_root == base / "linked-src":
                    linked_root.symlink_to(source, target_is_directory=True)
                output = io.StringIO()
                with (
                    patch.object(checker, "RUST_ROOT", linked_root),
                    patch.object(checker, "PLATFORM_ROOT", linked_root / "platform"),
                    contextlib.redirect_stdout(output),
                ):
                    status = checker.main()
                self.assertEqual(status, 1)
                self.assertIn("source root traverses a symlink", output.getvalue())
                self.assertNotIn("PRIVATE_FIXTURE", output.getvalue())

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
