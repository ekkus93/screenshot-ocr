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
            self.assertEqual(entries["src-tauri/Cargo.toml"]["owner"], "cargo_manifest")
            self.assertEqual(entries["src-tauri/src/app.rs"]["lines"]["desktop_environment"], [1])
            self.assertNotIn("src/node_modules/vendor.ts", entries)
            self.assertNotIn("docs/ignored.md", entries)
            self.assertNotIn("PRIVATE_FIXTURE", str(report))
            self.assertEqual(report, inventory.inventory(root))

    def test_portable_source_has_no_findings(self):
        with tempfile.TemporaryDirectory() as temp:
            source = Path(temp) / "src-tauri/src/app.rs"
            source.parent.mkdir(parents=True)
            source.write_text("fn portable() {}\n", encoding="utf-8")
            result = inventory.inventory(Path(temp))
            self.assertEqual(result["entries"], [])
            self.assertTrue(all(count == 0 for count in result["counts"].values()))


if __name__ == "__main__":
    unittest.main()
