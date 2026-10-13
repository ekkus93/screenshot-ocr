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
            self.assertEqual(entries[".github/actions/cross-platform/action.yml"]["owner"], "ci")
            self.assertIn("package.json", entries)
            self.assertNotIn("package-lock.json", entries)
            self.assertNotIn("pnpm-lock.yaml", entries)
            self.assertNotIn("PRIVATE_FIXTURE", str(report))
            self.assertEqual(report, inventory.inventory(root))

    def test_executable_inventory_covers_cross_platform_absolute_paths(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            source = (
                'let unix = "/usr/sbin/helper";\n'
                'let local = "/opt/vendor/bin/ocr";\n'
                'let windows = "C:/Program Files/OCR/tool";\n'
                r'let unc = r"\\server\share\tool";' + "\n"
                r'let system = r"System32\tool";' + "\n"
            )
            path = root / "src-tauri/src/paths.rs"
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(source, encoding="utf-8")
            report = inventory.inventory(root)
            entry = next(item for item in report["entries"] if item["path"] == "src-tauri/src/paths.rs")
            self.assertEqual(entry["lines"]["executable_path"], [1, 2, 3, 4, 5])
            self.assertNotIn("Program Files", str(report))
            self.assertEqual(report, inventory.inventory(root))

    def test_cargo_configuration_is_included_and_owned_by_build_metadata(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            config = root / ".cargo" / "config.toml"
            config.parent.mkdir(parents=True)
            config.write_text(
                '[target.x86_64-pc-windows-msvc]\n'
                'rustflags = ["-C", "link-arg=/NODEFAULTLIB"]\n',
                encoding="utf-8",
            )
            report = inventory.inventory(root)
            entries = {entry["path"]: entry for entry in report["entries"]}
            self.assertEqual(entries[".cargo/config.toml"]["owner"], "cargo_manifest")
            self.assertEqual(entries[".cargo/config.toml"]["lines"]["windows_native"], [1])
            self.assertNotIn("link-arg=/NODEFAULTLIB", str(report))

    def test_linked_cargo_config_is_rejected_without_reading_external_file(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp) / "repo"
            cargo = root / ".cargo"
            cargo.mkdir(parents=True)
            external = Path(temp) / "external.toml"
            external.write_text("PRIVATE_FIXTURE Windows", encoding="utf-8")
            (cargo / "config.toml").symlink_to(external)
            with self.assertRaisesRegex(ValueError, "first-party source traverses a symbolic link") as found:
                inventory.inventory(root)
            self.assertNotIn("PRIVATE_FIXTURE", str(found.exception))

    def test_symlinked_source_files_are_not_scanned(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp) / "repo"
            scripts = root / "scripts"
            scripts.mkdir(parents=True)
            outside = Path(temp) / "private.py"
            outside.write_text("GNOME PRIVATE_FIXTURE\\n", encoding="utf-8")
            (scripts / "linked.py").symlink_to(outside)
            with self.assertRaisesRegex(
                ValueError, "first-party source traverses a symbolic link"
            ) as found:
                inventory.inventory(root)
            self.assertNotIn("PRIVATE_FIXTURE", str(found.exception))

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
            with self.assertRaisesRegex(
                ValueError, "first-party source traverses a symbolic link"
            ) as found:
                inventory.inventory(root)
            self.assertNotIn("PRIVATE_FIXTURE", str(found.exception))

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

    def test_dangling_directory_link_cannot_hide_first_party_inventory(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp) / "repo"
            source = root / "src" / "platform"
            source.mkdir(parents=True)
            (source / "missing-source-tree").symlink_to(
                Path(temp) / "not-present", target_is_directory=True
            )
            with self.assertRaisesRegex(
                ValueError, "first-party source traverses a symbolic link"
            ):
                inventory.inventory(root)

    def test_unrecognized_source_link_is_rejected_without_reading_target(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp) / "repo"
            source = root / "scripts"
            source.mkdir(parents=True)
            target = Path(temp) / "external.data"
            target.write_text("PRIVATE_FIXTURE", encoding="utf-8")
            (source / "fixture.data").symlink_to(target)
            with self.assertRaisesRegex(
                ValueError, "first-party source traverses a symbolic link"
            ) as found:
                inventory.inventory(root)
            self.assertNotIn("PRIVATE_FIXTURE", str(found.exception))

    def test_root_level_linked_source_is_rejected_without_following_target(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp) / "repo"
            root.mkdir()
            target = Path(temp) / "external.ts"
            target.write_text("GNOME PRIVATE_FIXTURE\\n", encoding="utf-8")
            (root / "vite.config.ts").symlink_to(target)
            with self.assertRaisesRegex(
                ValueError, "first-party source traverses a symbolic link"
            ) as found:
                inventory.inventory(root)
            self.assertNotIn("PRIVATE_FIXTURE", str(found.exception))

    def test_linked_generated_dependencies_are_excluded(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp) / "repo"
            generated = root / "src" / "node_modules"
            generated.mkdir(parents=True)
            target = Path(temp) / "external"
            target.mkdir()
            (target / "fixture.ts").write_text(
                "GNOME PRIVATE_FIXTURE\\n", encoding="utf-8"
            )
            (generated / "linked").symlink_to(target, target_is_directory=True)
            report = inventory.inventory(root)
            self.assertEqual(report["entries"], [])
            self.assertNotIn("PRIVATE_FIXTURE", str(report))

    def test_shortcut_aliases_and_platform_resource_locations_are_inventory_visible(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            source = (
                'let linux = "Super+Shift+O";\n'
                'let windows = "Ctrl+Shift+O";\n'
                'let tauri = "CmdOrCtrl+Shift+O";\n'
                'let alias = "CommandOrControl+Shift+O";\n'
                'let win = "Win+Shift+O";\n'
                'let private = app_local_data_dir();\n'
                'let scratch = std::env::temp_dir();\n'
                'let known = SHGetKnownFolderPath(FOLDERID_LocalAppData);\n'
            )
            path = root / "src-tauri/src/assumptions.rs"
            path.parent.mkdir(parents=True)
            path.write_text(source, encoding="utf-8")
            report = inventory.inventory(root)
            entry = next(item for item in report["entries"] if item["path"] == "src-tauri/src/assumptions.rs")
            self.assertEqual(entry["lines"]["shortcut"], [1, 2, 3, 4, 5])
            self.assertEqual(entry["lines"]["platform_resource"], [6, 7, 8])
            self.assertNotIn("LocalAppData", str(report))
            self.assertEqual(report, inventory.inventory(root))

    def test_process_discovery_dll_loading_and_os_path_probes_are_visible(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            source = (
                'let child = Command::new("unreviewed-helper");\\n'
                'let helper = which::which("helper");\\n'
                'let library = LoadLibraryExW(name, 0, flags);\\n'
                'SetDefaultDllDirectories(flags);\\n'
                'let path = SHGetFolderPathW(hwnd, id, 0, 0, out);\\n'
                'let os = std::env::consts::OS;\\n'
                'let cfg = dirs::config_dir();\\n'
            )
            path = root / "src-tauri/src/discovery.rs"
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(source, encoding="utf-8")
            report = inventory.inventory(root)
            entry = next(item for item in report["entries"] if item["path"] == "src-tauri/src/discovery.rs")
            self.assertEqual(entry["lines"]["process_discovery"], [1, 2])
            self.assertEqual(entry["lines"]["dll_loader"], [3, 4])
            self.assertEqual(entry["lines"]["os_path_probe"], [5, 6, 7])
            self.assertNotIn("unreviewed-helper", str(report))
            self.assertEqual(report, inventory.inventory(root))

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
