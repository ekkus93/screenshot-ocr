"""Synthetic, content-free tests for the Windows PE import inventory."""
from __future__ import annotations

import hashlib
import importlib.util
import struct
import tempfile
import unittest
from pathlib import Path

spec = importlib.util.spec_from_file_location(
    "windows_pe_inventory", Path(__file__).with_name("audit-windows-pe-imports.py")
)
assert spec and spec.loader
audit = importlib.util.module_from_spec(spec)
spec.loader.exec_module(audit)


def fixture(*, pe64: bool = True, delay: bool = False) -> bytes:
    data = bytearray(0x600)
    data[:2] = b"MZ"
    struct.pack_into("<I", data, 0x3C, 0x80)
    data[0x80:0x84] = b"PE\0\0"
    struct.pack_into("<H", data, 0x84, 0x8664 if pe64 else 0x14C)
    struct.pack_into("<H", data, 0x86, 1)  # section count
    size = 240 if pe64 else 224
    struct.pack_into("<H", data, 0x94, size)
    opt = 0x98
    struct.pack_into("<H", data, opt, 0x20B if pe64 else 0x10B)
    directory = opt + (112 if pe64 else 96)
    if pe64:
        struct.pack_into("<Q", data, opt + 24, 0x140000000)
    else:
        struct.pack_into("<I", data, opt + 28, 0x400000)
    struct.pack_into("<II", data, directory + 8, 0x1000, 40)
    section = opt + size
    struct.pack_into("<IIII", data, section + 8, 0x300, 0x1000, 0x300, 0x200)
    struct.pack_into("<IIIII", data, 0x200, 0, 0, 0, 0x1050, 0)
    data[0x250:0x25D] = b"KERNEL32.dll\0"
    if delay:
        struct.pack_into("<II", data, directory + 13 * 8, 0x1100, 64)
        struct.pack_into("<II", data, 0x300, 1, 0x1140)
        data[0x340:0x34F] = b"vendor-extra.dll\0"
    return bytes(data)


class PeImportTests(unittest.TestCase):
    def test_import_directory_works_for_pe32_and_pe32_plus(self):
        for pe64 in (True, False):
            with self.subTest(pe64=pe64):
                self.assertEqual(audit.imports(fixture(pe64=pe64)), {"kernel32.dll"})

    def test_delay_import_is_included(self):
        self.assertEqual(
            audit.imports(fixture(delay=True)),
            {"kernel32.dll", "vendor-extra.dll"},
        )

    def test_optional_header_cannot_overlap_section_table(self):
        for pe64, minimum in ((True, 112), (False, 96)):
            for size in (0, 1, minimum - 1):
                with self.subTest(pe64=pe64, size=size):
                    data = bytearray(fixture(pe64=pe64))
                    struct.pack_into("<H", data, 0x94, size)
                    with self.assertRaisesRegex(audit.InvalidPE, "optional header"):
                        audit.imports(bytes(data))

    def test_import_directory_size_without_rva_is_rejected(self):
        for delay in (False, True):
            with self.subTest(delay=delay):
                data = bytearray(fixture(delay=delay))
                opt = 0x98
                directory = opt + 112
                entry = directory + (13 if delay else 1) * 8
                struct.pack_into("<II", data, entry, 0, 20)
                with self.assertRaisesRegex(audit.InvalidPE, "size without RVA"):
                    audit.imports(bytes(data))

    def test_truncated_and_invalid_pe_fail_closed(self):
        for invalid in (b"", b"MZ", b"not a PE", fixture()[:0x150]):
            with self.subTest(length=len(invalid)):
                with self.assertRaises(audit.InvalidPE):
                    audit.imports(invalid)

    def test_import_names_cannot_inject_paths_or_log_lines(self):
        for invalid in (
            b"bad:name.dll\0",
            b"bad\nname.dll\0",
            b"bad name.dll\0",
            b"bad;name.dll\0",
        ):
            with self.subTest(invalid=invalid):
                data = bytearray(fixture())
                data[0x250 : 0x250 + len(invalid)] = invalid
                with self.assertRaisesRegex(audit.InvalidPE, "invalid DLL name"):
                    audit.imports(bytes(data))

    def test_audit_identifies_packaged_and_external_edges(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "tesseract.exe").write_bytes(fixture(delay=True))
            (root / "vendor-extra.dll").write_bytes(fixture())
            result = audit.audit(root)
            self.assertEqual(len(result["binaries"]), 2)
            self.assertTrue(all(row["machine"] == "AMD64" for row in result["binaries"]))
            for row in result["binaries"]:
                payload = (root / row["binary"]).read_bytes()
                self.assertEqual(row["sha256"], hashlib.sha256(payload).hexdigest())
                self.assertEqual(row["size_bytes"], len(payload))
            self.assertEqual(result["binaries"][0]["packaged"], ["vendor-extra.dll"])
            self.assertIn("kernel32.dll", result["external_unreviewed"])
            self.assertEqual(result["qualification"].startswith("static PE"), True)


    def test_uppercase_dll_extension_is_included_on_all_hosts(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "tesseract.exe").write_bytes(fixture(delay=True))
            (root / "vendor-extra.DLL").write_bytes(fixture())
            result = audit.audit(root)
            self.assertEqual(len(result["binaries"]), 2)
            self.assertEqual(result["binaries"][0]["packaged"], ["vendor-extra.dll"])
            self.assertEqual(
                result["statically_reachable_binaries"],
                ["tesseract.exe", "vendor-extra.dll"],
            )

    def test_audit_rejects_x86_executable_or_dependency(self):
        for x86_executable in (True, False):
            with self.subTest(x86_executable=x86_executable):
                with tempfile.TemporaryDirectory() as tmp:
                    root = Path(tmp)
                    (root / "tesseract.exe").write_bytes(
                        fixture(pe64=not x86_executable)
                    )
                    (root / "vendor-extra.dll").write_bytes(
                        fixture(pe64=x86_executable)
                    )
                    with self.assertRaisesRegex(audit.InvalidPE, "AMD64"):
                        audit.audit(root)

    def test_audit_rejects_amd64_machine_with_pe32_header(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            mismatched = bytearray(fixture(pe64=False))
            struct.pack_into("<H", mismatched, 0x84, 0x8664)
            (root / "tesseract.exe").write_bytes(mismatched)
            (root / "vendor-extra.dll").write_bytes(fixture())
            with self.assertRaisesRegex(audit.InvalidPE, "AMD64"):
                audit.audit(root)

    def test_static_reachability_prioritizes_without_claiming_unused_dlls(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            # The executable delay-imports vendor-extra; an additional DLL
            # is not statically reachable but might be loaded dynamically.
            (root / "tesseract.exe").write_bytes(fixture(delay=True))
            (root / "vendor-extra.dll").write_bytes(fixture())
            (root / "optional-plugin.dll").write_bytes(fixture())
            result = audit.audit(root)
            self.assertEqual(
                result["statically_reachable_binaries"],
                ["tesseract.exe", "vendor-extra.dll"],
            )
            self.assertEqual(
                result["not_statically_reachable_binaries"],
                ["optional-plugin.dll"],
            )
            self.assertEqual(
                result["external_unreviewed_from_statically_reachable"],
                ["kernel32.dll"],
            )
            self.assertIn("dynamic loads", result["qualification"])


if __name__ == "__main__":
    unittest.main()
