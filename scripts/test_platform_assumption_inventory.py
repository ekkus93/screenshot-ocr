"""Synthetic content-privacy and classification tests for platform inventory."""
from __future__ import annotations

import importlib.util
import tempfile
import unittest
from pathlib import Path

SPEC = importlib.util.spec_from_file_location(
    "platform_assumption_inventory", Path(__file__).with_name("audit-platform-assumptions.py")
)
assert SPEC is not None and SPEC.loader is not None
inventory = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(inventory)


class AssumptionInventoryTests(unittest.TestCase):
    def test_classifies_sources_without_emitting_sensitive_text(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            for filename, source in {
                "src-tauri/src/app.rs": 'let secret = "WAYLAND_DISPLAY PRIVATE_FIXTURE";\n',
                "src-tauri/src/platform/linux/capture/portal.rs": "use ashpd::desktop;\n",
                "src-tauri/src/platform/windows/selector.rs": "use windows_sys::Win32::Foundation;\n",
                "src-tauri/src/platform.rs": "use WindowsPlatform;\n",
                "src-tauri/src/platform/mod.rs": "use WindowsPlatform;\n",
                "src-tauri/src/platform/services.rs": "use WindowsPlatform;\n",
                "src-tauri/Cargo.toml": 'windows-sys = "0.61"\n',
                "src/node_modules/vendor.ts": "GNOME PRIVATE_FIXTURE\n",
                "docs/ignored.md": "PRIVATE_FIXTURE GNOME\n",
            }.items():
                dest = root / filename
                dest.parent.mkdir(parents=True, exist_ok=True)
                dest.write_text(source, encoding="utf-8")
            report = inventory.inventory(root)
            entries = {item["path"]: item for item in report["entries"]}
            self.assertEqual(entries["src-tauri/src/app.rs"]["owner"], "shared_core_or_desktop")
            self.assertEqual(entries["src-tauri/src/platform/linux/capture/portal.rs"]["owner"], "linux_adapter")
            self.assertEqual(entries["src-tauri/src/platform/windows/selector.rs"]["owner"], "windows_adapter")
            self.assertEqual(entries["src-tauri/src/platform.rs"]["owner"], "platform_composition")
            self.assertEqual(entries["src-tauri/src/platform/mod.rs"]["owner"], "platform_composition")
            self.assertEqual(entries["src-tauri/src/platform/services.rs"]["owner"], "platform_contract")
            self.assertEqual(entries["src-tauri/Cargo.toml"]["owner"], "cargo_manifest")
            self.assertEqual(entries["src-tauri/src/app.rs"]["lines"]["desktop_environment"], [1])
            self.assertNotIn("src/node_modules/vendor.ts", entries)
            self.assertNotIn("docs/ignored.md", entries)
            self.assertNotIn("PRIVATE_FIXTURE", str(report))
            self.assertEqual(report, inventory.inventory(root))

    def test_installer_metadata_and_native_windows_paths_are_in_scope(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            fixtures = {
                "src-tauri/tauri.conf.json": '{"bundle":{"targets":["deb"]}}\\n',
                "src-tauri/tauri.windows.conf.json": '{"bundle":{"targets":["nsis"]}}\\n',
                "src-tauri/build.rs": 'let registry = "HKEY_CURRENT_USER";\\n',
                "src-tauri/installer/windows.nsh": 'StrCpy $0 "$PROGRAMFILES"\\n',
                "scripts/install.sh": 'exec /usr/bin/gnome-screenshot\\n',
                "src-tauri/target/generated.rs": 'WindowsPlatform SECRET_FIXTURE\\n',
            }
            for name, source in fixtures.items():
                path = root / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(source, encoding="utf-8")
            report = inventory.inventory(root)
            entries = {entry["path"]: entry for entry in report["entries"]}
            self.assertEqual(entries["src-tauri/tauri.conf.json"]["lines"]["bundle_target"], [1])
            self.assertEqual(entries["src-tauri/tauri.windows.conf.json"]["lines"]["bundle_target"], [1])
            self.assertEqual(entries["src-tauri/build.rs"]["lines"]["windows_native"], [1])
            self.assertEqual(entries["src-tauri/installer/windows.nsh"]["lines"]["windows_environment"], [1])
            self.assertEqual(entries["scripts/install.sh"]["lines"]["linux_gnome"], [1])
            self.assertNotIn("src-tauri/target/generated.rs", entries)
            self.assertNotIn("SECRET_FIXTURE", str(report))

    def test_root_build_configs_and_reusable_actions_are_in_scope(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            fixtures = {
                "vite.config.ts": 'const binary = "C:\\Program Files\\OCR\\tesseract.exe";\n',
                "playwright.config.ts": 'const shortcut = "Control+Shift+O";\n',
                ".github/actions/cross-platform/action.yml": 'run: gnome-screenshot\n',
                "package.json": '{"scripts":{"windows":"check"}}\n',
                "package-lock.json": "GNOME PRIVATE_FIXTURE\n",
                "pnpm-lock.yaml": "GNOME PRIVATE_FIXTURE\n",
                "src/ordinary.ts": "export const portable = true;\n",
            }
            for name, content in fixtures.items():
                path = root / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(content, encoding="utf-8")
            report = inventory.inventory(root)
            entries = {entry["path"]: entry for entry in report["entries"]}
            self.assertEqual(entries["vite.config.ts"]["lines"]["executable_path"], [1])
            self.assertEqual(entries["playwright.config.ts"]["lines"]["shortcut"], [1])
            self.assertEqual(
                entries[".github/actions/cross-platform/action.yml"]["lines"]["linux_gnome"], [1]
            )
            self.assertIn("package.json", entries)
            self.assertNotIn("package-lock.json", entries)
            self.assertNotIn("pnpm-lock.yaml", entries)
            self.assertNotIn("PRIVATE_FIXTURE", str(report))
            self.assertEqual(report, inventory.inventory(root))

    def test_symlinked_source_files_are_not_scanned(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp) / "repo"
            scripts = root / "scripts"
            scripts.mkdir(parents=True)
            outside = Path(temp) / "private.py"
            outside.write_text("GNOME PRIVATE_FIXTURE\\n", encoding="utf-8")
            (scripts / "linked.py").symlink_to(outside)
            report = inventory.inventory(root)
            self.assertEqual(report["entries"], [])
            self.assertNotIn("PRIVATE_FIXTURE", str(report))

    def test_symlinked_source_directories_are_not_scanned(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp) / "repo"
            root.mkdir()
            outside = Path(temp) / "private"
            outside.mkdir()
            (outside / "leak.rs").write_text(
                "GNOME PRIVATE_FIXTURE\\n", encoding="utf-8"
            )
            # A top-level scan root may itself be a symlink, even when its
            # child is an ordinary file. A nested directory link is likewise
            # not part of the checkout's first-party source inventory.
            (root / "src-tauri").symlink_to(outside, target_is_directory=True)
            (root / "scripts").mkdir()
            (root / "scripts" / "linked").symlink_to(
                outside, target_is_directory=True
            )
            with self.assertRaisesRegex(ValueError, "source tree traverses a symbolic link"):
                inventory.inventory(root)
            (root / "src-tauri").unlink()
            report = inventory.inventory(root)
            self.assertEqual(report["entries"], [])
            self.assertNotIn("PRIVATE_FIXTURE", str(report))

    def test_symlinked_checkout_parent_fails_closed(self):
        with tempfile.TemporaryDirectory() as temp:
            actual = Path(temp) / "actual"
            checkout = actual / "repo"
            (checkout / "src").mkdir(parents=True)
            (checkout / "src" / "fixture.ts").write_text("GNOME fixture\n", encoding="utf-8")
            alias = Path(temp) / "alias"
            alias.symlink_to(actual, target_is_directory=True)
            self.assertFalse((alias / "repo").is_symlink())
            with self.assertRaisesRegex(ValueError, "source root must not be a symlink"):
                inventory.inventory(alias / "repo")

    def test_portable_source_has_no_findings(self):
        with tempfile.TemporaryDirectory() as temp:
            source = Path(temp) / "src-tauri/src/app.rs"
            source.parent.mkdir(parents=True)
            source.write_text("fn portable() {}\n", encoding="utf-8")
            result = inventory.inventory(Path(temp))
            self.assertEqual(result["entries"], [])
            self.assertTrue(all(count == 0 for count in result["counts"].values()))


    def test_windows_build_conditions_native_bindings_and_resource_files(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            fixtures = {
                "src-tauri/src/cfg.rs": (
                    "let enabled = cfg!(windows);\n"
                    "#[cfg(unix)]\n"
                    '#[cfg(target_family = "windows")]\n'
                    "use std::os::windows::ffi::OsStrExt;\n"
                ),
                "src-tauri/src/ffi.rs": (
                    "use winapi::um::winuser;\n"
                    "let error = std::io::Error::last_os_error();\n"
                ),
                "src-tauri/installer/version.rc": 'VALUE "FileDescription", "Windows app"\n',
                "scripts/launch.cmd": "set OCR=C:\\Program Files\\OCR\\tesseract.exe\n",
                "src-tauri/target/generated.cmd": "GNOME PRIVATE_FIXTURE\n",
            }
            for name, content in fixtures.items():
                path = root / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(content, encoding="utf-8")
            report = inventory.inventory(root)
            entries = {entry["path"]: entry for entry in report["entries"]}
            self.assertEqual(
                entries["src-tauri/src/cfg.rs"]["lines"]["target_conditional"],
                [1, 2, 3, 4],
            )
            self.assertEqual(entries["src-tauri/src/ffi.rs"]["lines"]["native_binding"], [1])
            self.assertEqual(entries["src-tauri/src/ffi.rs"]["lines"]["native_error"], [2])
            self.assertEqual(entries["scripts/launch.cmd"]["lines"]["executable_path"], [1])
            self.assertEqual(entries["src-tauri/installer/version.rc"]["lines"]["windows_native"], [1])
            self.assertNotIn("src-tauri/target/generated.cmd", entries)
            self.assertNotIn("PRIVATE_FIXTURE", str(report))
            self.assertEqual(report, inventory.inventory(root))

    def test_symlinked_checkout_root_fails_closed(self):
        with tempfile.TemporaryDirectory() as temp:
            outside = Path(temp) / "private"
            (outside / "src").mkdir(parents=True)
            (outside / "src" / "secret.ts").write_text(
                "GNOME PRIVATE_FIXTURE\n", encoding="utf-8"
            )
            linked_root = Path(temp) / "linked-repo"
            linked_root.symlink_to(outside, target_is_directory=True)
            with self.assertRaisesRegex(ValueError, "source root must not be a symlink"):
                inventory.inventory(linked_root)

if __name__ == "__main__":
    unittest.main()
