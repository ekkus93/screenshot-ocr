"""Synthetic, content-free tests for the Windows PE import inventory."""
from __future__ import annotations

import hashlib
import importlib.util
import struct
import tempfile
import unittest
from types import SimpleNamespace
from unittest.mock import patch
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
    struct.pack_into("<I", data, opt + (108 if pe64 else 92), 16)
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

    def test_directory_count_controls_import_and_delay_tables(self):
        for pe64 in (True, False):
            for count, expected in (
                (1, set()),
                (2, {"kernel32.dll"}),
                (14, {"kernel32.dll", "vendor-extra.dll"}),
            ):
                with self.subTest(pe64=pe64, count=count):
                    data = bytearray(fixture(pe64=pe64, delay=True))
                    struct.pack_into(
                        "<I", data, 0x98 + (108 if pe64 else 92), count
                    )
                    self.assertEqual(audit.imports(bytes(data)), expected)

    def test_directory_count_must_fit_optional_header(self):
        for pe64 in (True, False):
            with self.subTest(pe64=pe64):
                data = bytearray(fixture(pe64=pe64))
                struct.pack_into("<I", data, 0x98 + (108 if pe64 else 92), 17)
                with self.assertRaisesRegex(
                    audit.InvalidPE, "data-directory count exceeds"
                ):
                    audit.imports(bytes(data))

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

    def test_import_descriptor_table_must_terminate_within_declared_size(self):
        for delay in (False, True):
            with self.subTest(delay=delay):
                data = bytearray(fixture(delay=delay))
                directory = 0x98 + 112
                index = 13 if delay else 1
                record_size = 32 if delay else 20
                # The first descriptor is valid, but its null terminator
                # lives just outside the advertised data-directory size.
                struct.pack_into(
                    "<I", data, directory + index * 8 + 4, record_size
                )
                with self.assertRaisesRegex(
                    audit.InvalidPE, "unterminated import descriptor"
                ):
                    audit.imports(bytes(data))
                # A directory shorter than one descriptor is invalid too.
                struct.pack_into(
                    "<I", data, directory + index * 8 + 4, record_size - 1
                )
                with self.assertRaisesRegex(
                    audit.InvalidPE, "import directory too small"
                ):
                    audit.imports(bytes(data))

    def test_truncated_and_invalid_pe_fail_closed(self):
        for invalid in (b"", b"MZ", b"not a PE", fixture()[:0x150]):
            with self.subTest(length=len(invalid)):
                with self.assertRaises(audit.InvalidPE):
                    audit.imports(invalid)

    def test_dll_name_must_terminate_inside_file_backed_section(self):
        data = bytearray(fixture())
        # The DLL name starts at raw offset 0x250, but its terminator is
        # beyond this shortened section's 0x258 file-backed boundary.
        section = 0x98 + 240
        struct.pack_into("<I", data, section + 16, 0x58)
        with self.assertRaisesRegex(audit.InvalidPE, "unterminated DLL name"):
            audit.imports(bytes(data))

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


    def test_audit_rejects_symlinked_parent_of_staging_directory(self):
        with tempfile.TemporaryDirectory() as tmp:
            base = Path(tmp)
            owned = base / "owned"
            runtime = owned / "runtime"
            runtime.mkdir(parents=True)
            (runtime / "tesseract.exe").write_bytes(fixture())
            (runtime / "vendor-extra.dll").write_bytes(fixture())
            linked = base / "linked"
            try:
                linked.symlink_to(owned, target_is_directory=True)
            except (OSError, NotImplementedError):
                self.skipTest("host does not permit directory symlinks")
            with self.assertRaisesRegex(audit.InvalidPE, "directly owned directory"):
                audit.audit(linked / "runtime")

    def test_relative_staging_root_rejects_linked_ancestor(self):
        with tempfile.TemporaryDirectory() as tmp:
            base = Path(tmp)
            real = base / "owned" / "runtime"
            real.mkdir(parents=True)
            (real / "tesseract.exe").write_bytes(fixture())
            (real / "vendor-extra.dll").write_bytes(fixture())
            linked = base / "linked"
            try:
                linked.symlink_to(base / "owned", target_is_directory=True)
            except (OSError, NotImplementedError):
                self.skipTest("host does not permit directory symlinks")
            # Relative path must be checked through lexical absolute ancestry,
            # not only its immediate root spelling.
            relative = Path("linked") / "runtime"
            old_cwd = Path.cwd()
            try:
                import os
                os.chdir(base)
                with self.assertRaisesRegex(audit.InvalidPE, "directly owned directory"):
                    audit.audit(relative)
            finally:
                os.chdir(old_cwd)

    def test_windows_reparse_attribute_is_detected_without_following_link(self):
        # Windows junctions can have FILE_ATTRIBUTE_REPARSE_POINT without
        # Path.is_symlink() being true. Exercise that metadata on any host.
        fake = SimpleNamespace(st_file_attributes=0x400, st_mode=0o100644)
        with patch.object(Path, "lstat", return_value=fake):
            self.assertTrue(audit.is_redirected(Path("synthetic.dll")))

    def test_audit_rejects_reparse_pointed_dll(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "tesseract.exe").write_bytes(fixture())
            (root / "vendor-extra.dll").write_bytes(fixture())
            real_check = audit.is_redirected
            with patch.object(audit, "is_redirected", side_effect=lambda p: p.name == "vendor-extra.dll" or real_check(p)):
                with self.assertRaisesRegex(audit.InvalidPE, "redirected file"):
                    audit.audit(root)

    def test_audit_rejects_log_delimiters_in_staged_dll_filename(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "tesseract.exe").write_bytes(fixture())
            (root / "bad;name.dll").write_bytes(fixture())
            with self.assertRaisesRegex(audit.InvalidPE, "invalid staged OCR binary name"):
                audit.audit(root)

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

    def test_preliminary_native_review_register_rejects_identity_drift(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "tesseract.exe").write_bytes(fixture(delay=True))
            (root / "vendor-extra.dll").write_bytes(fixture())
            report = audit.audit(root)
            q = chr(96)
            lines = ["## Packaged binary review register"]
            reachable = set(report["statically_reachable_binaries"])
            for row in report["binaries"]:
                yes = "YES" if row["binary"] in reachable else "NO"
                lines.append(
                    f'| {q}{row["binary"]}{q} | {yes} | {row["size_bytes"]} '
                    f'| {q}{row["sha256"]}{q} | UNREVIEWED |'
                )
            lines += ["## External imports still unreviewed", "All external basenames:"]
            lines += [f"- {q}{name}{q}" for name in report["external_unreviewed"]]
            lines += ["External basenames from statically reachable binaries:"]
            lines += [
                f"- {q}{name}{q}"
                for name in report["external_unreviewed_from_statically_reachable"]
            ]
            content = "\n".join(lines)
            audit.verify_review_register(report, content)
            modified = content.replace(report["binaries"][0]["sha256"], "0" * 64)
            with self.assertRaisesRegex(audit.InvalidPE, "bytes differ"):
                audit.verify_review_register(report, modified)
            modified = content.replace(" | YES | ", " | NO | ", 1)
            with self.assertRaisesRegex(audit.InvalidPE, "reachability differs"):
                audit.verify_review_register(report, modified)
            modified = content.replace("All external basenames:", "External imports:")
            with self.assertRaisesRegex(audit.InvalidPE, "incomplete"):
                audit.verify_review_register(report, modified)

    def test_review_register_accepts_mixed_case_native_dll_identity(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "tesseract.exe").write_bytes(fixture(delay=True))
            # The actual pinned distribution contains libLerc.dll.
            (root / "Vendor-Extra.DLL").write_bytes(fixture())
            report = audit.audit(root)
            q = chr(96)
            reachable = set(report["statically_reachable_binaries"])
            rows = ["## Packaged binary review register"]
            for record in report["binaries"]:
                name = record["binary"]
                yes = "YES" if name.lower() in reachable else "NO"
                rows.append(
                    f'| {q}{name}{q} | {yes} | {record["size_bytes"]} '
                    f'| {q}{record["sha256"]}{q} | UNREVIEWED |'
                )
            rows.extend(["## External imports still unreviewed", "All external basenames:"])
            rows.extend(f"- {q}{name}{q}" for name in report["external_unreviewed"])
            rows.append("External basenames from statically reachable binaries:")
            rows.extend(
                f"- {q}{name}{q}"
                for name in report["external_unreviewed_from_statically_reachable"]
            )
            content = "\n".join(rows)
            audit.verify_review_register(report, content)
            duplicate = content.replace(
                "## External imports still unreviewed",
                rows[2].replace("Vendor-Extra.DLL", "vendor-extra.dll")
                + "\n## External imports still unreviewed",
            )
            with self.assertRaisesRegex(audit.InvalidPE, "invalid native review identity"):
                audit.verify_review_register(report, duplicate)

    def test_review_gate_errors_are_diagnostic_but_content_free(self):
        for reason in audit.SAFE_REVIEW_FAILURES:
            self.assertIn(reason, audit.public_audit_failure(audit.InvalidPE(reason)))
        for error in (
            audit.InvalidPE("C:\\Users\\Private\\secret.dll"),
            OSError("C:\\Users\\Private\\secret.dll"),
            audit.InvalidPE("unexpected user text"),
        ):
            public = audit.public_audit_failure(error)
            self.assertEqual(
                public,
                "Windows PE dependency audit failed: malformed or inaccessible staged payload.",
            )
            self.assertNotIn("secret", public)
            self.assertNotIn("Private", public)

    def test_audit_rejects_unreviewed_or_redirected_runtime_layout(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "tesseract.exe").write_bytes(fixture())
            (root / "vendor-extra.dll").write_bytes(fixture())
            self.assertEqual(len(audit.audit(root)["binaries"]), 2)

            unrelated = root / "unexpected.txt"
            unrelated.write_text("synthetic fixture", encoding="utf-8")
            with self.assertRaises(audit.InvalidPE):
                audit.audit(root)
            unrelated.unlink()

            tessdata = root / "tessdata"
            tessdata.mkdir()
            english = tessdata / "eng.traineddata"
            english.write_bytes(b"synthetic fixture")
            self.assertEqual(len(audit.audit(root)["binaries"]), 2)

            extra = tessdata / "unexpected.traineddata"
            extra.write_bytes(b"synthetic fixture")
            with self.assertRaises(audit.InvalidPE):
                audit.audit(root)
            extra.unlink()

            # Reject directory impersonation even when its extension looks
            # like an adjacent DLL, instead of silently ignoring it.
            fake_dll = root / "imposter.dll"
            fake_dll.mkdir()
            with self.assertRaises(audit.InvalidPE):
                audit.audit(root)
            fake_dll.rmdir()

            outside = Path(tmp).parent / ("linked-" + root.name)
            try:
                tessdata.rmdir()
            except OSError:
                english.unlink()
                tessdata.rmdir()
            try:
                tessdata.symlink_to(root, target_is_directory=True)
            except (OSError, NotImplementedError):
                return  # Symlink creation may require privileges on Windows.
            with self.assertRaises(audit.InvalidPE):
                audit.audit(root)

    def test_staged_language_data_must_be_exactly_one_nonempty_regular_file(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "tesseract.exe").write_bytes(fixture())
            (root / "vendor-extra.dll").write_bytes(fixture())
            # Native-PE-only fixtures may omit tessdata entirely.
            self.assertEqual(len(audit.audit(root)["binaries"]), 2)

            tessdata = root / "tessdata"
            tessdata.mkdir()
            with self.assertRaisesRegex(audit.InvalidPE, "unreviewed language files"):
                audit.audit(root)

            english = tessdata / "eng.traineddata"
            english.write_bytes(b"")
            with self.assertRaisesRegex(audit.InvalidPE, "invalid English language data"):
                audit.audit(root)
            english.write_bytes(b"synthetic language model fixture")
            self.assertEqual(len(audit.audit(root)["binaries"]), 2)

            # A case-only duplicate language filename can exist on some
            # case-sensitive Windows/NTFS configurations.
            alias = tessdata / "ENG.TRAINEDDATA"
            if not alias.exists():
                alias.write_bytes(b"synthetic duplicate")
                with self.assertRaisesRegex(audit.InvalidPE, "unreviewed language files"):
                    audit.audit(root)
                alias.unlink()

            wrong = tessdata / "unreviewed.traineddata"
            english.unlink()
            wrong.write_bytes(b"synthetic fixture")
            with self.assertRaisesRegex(audit.InvalidPE, "invalid English language data"):
                audit.audit(root)

    def test_runtime_paths_require_exact_application_resource_spelling(self):
        # The Windows app constructs exact lowercase resource paths. On
        # case-sensitive NTFS an uppercase spelling is a different file.
        for candidate in ("TESSERACT.EXE", "TESSDATA", "ENG.TRAINEDDATA"):
            with self.subTest(candidate=candidate):
                with tempfile.TemporaryDirectory() as tmp:
                    root = Path(tmp)
                    binary = candidate if candidate == "TESSERACT.EXE" else "tesseract.exe"
                    tessdata = candidate if candidate == "TESSDATA" else "tessdata"
                    english = candidate if candidate == "ENG.TRAINEDDATA" else "eng.traineddata"
                    (root / binary).write_bytes(fixture())
                    (root / "vendor-extra.dll").write_bytes(fixture())
                    (root / tessdata).mkdir()
                    (root / tessdata / english).write_bytes(b"synthetic fixture")
                    with self.assertRaises(audit.InvalidPE):
                        audit.audit(root)

    def test_runtime_entry_names_reject_case_only_collisions(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "tesseract.exe").write_bytes(fixture())
            (root / "vendor-extra.dll").write_bytes(fixture())
            # On case-insensitive filesystems the second spelling is the
            # same entry and cannot create a collision; exercise both where
            # the filesystem supports distinct spellings.
            alias = root / "VENDOR-EXTRA.DLL"
            alias.write_bytes(fixture())
            if len(list(root.iterdir())) == 3:
                with self.assertRaisesRegex(audit.InvalidPE, "case-insensitive runtime entry collision"):
                    audit.audit(root)
            else:
                self.assertEqual(len(audit.audit(root)["binaries"]), 2)

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
