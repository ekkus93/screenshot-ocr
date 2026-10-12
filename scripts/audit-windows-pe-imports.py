#!/usr/bin/env python3
"""Inventory static and delay-loaded PE imports for staged Windows OCR binaries.

This is an evidence aid, NOT a redistribution license audit or an assertion
that externally provided DLLs are safe or available on a clean Windows host.
Only filenames are reported; no user content, paths, or binary bytes are logged.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import stat
import struct
from pathlib import Path


class InvalidPE(ValueError):
    """Malformed or unsupported PE import metadata."""


# Only literal, fixed review reasons are safe to show in CI logs. Raw OS
# errors, paths and unexpected parser exceptions remain fully suppressed.
SAFE_REVIEW_FAILURES = frozenset({
    "malformed native review row",
    "invalid native review identity",
    "invalid external native review row",
    "duplicate external native review row",
    "native review register incomplete",
    "staged native binary set differs from review",
    "staged native bytes differ from review",
    "staged native reachability differs from review",
    "staged native external imports differ from review",
})


def public_audit_failure(error: Exception) -> str:
    reason = str(error)
    if not isinstance(error, InvalidPE) or reason not in SAFE_REVIEW_FAILURES:
        reason = "malformed or inaccessible staged payload"
    return f"Windows PE dependency audit failed: {reason}."


def uint16(data: bytes, offset: int) -> int:
    if offset < 0 or offset + 2 > len(data):
        raise InvalidPE("truncated PE header")
    return struct.unpack_from("<H", data, offset)[0]


def uint32(data: bytes, offset: int) -> int:
    if offset < 0 or offset + 4 > len(data):
        raise InvalidPE("truncated PE header")
    return struct.unpack_from("<I", data, offset)[0]


def pe_machine(data: bytes) -> int:
    """Read the COFF machine type without trusting the input filename."""
    if len(data) < 0x40 or data[:2] != b"MZ":
        raise InvalidPE("invalid DOS signature")
    pe = uint32(data, 0x3C)
    if pe + 24 > len(data) or data[pe : pe + 4] != b"PE\0\0":
        raise InvalidPE("invalid PE signature")
    return uint16(data, pe + 4)


def imports(data: bytes) -> set[str]:
    if len(data) < 0x40 or data[:2] != b"MZ":
        raise InvalidPE("invalid DOS signature")
    pe = uint32(data, 0x3C)
    if pe + 24 > len(data) or data[pe : pe + 4] != b"PE\0\0":
        raise InvalidPE("invalid PE signature")
    count = uint16(data, pe + 6)
    optional_size = uint16(data, pe + 20)
    opt = pe + 24
    end = opt + optional_size
    if end > len(data):
        raise InvalidPE("truncated optional header")
    # Do not read a data-directory entry from the section table when a
    # malformed PE declares an undersized optional header. A directory can
    # only be absent when its entire layout is valid for the PE flavor.
    if optional_size < 2:
        raise InvalidPE("truncated optional header")
    magic = uint16(data, opt)
    if magic == 0x20B:  # PE32+
        if optional_size < 112:
            raise InvalidPE("truncated PE32+ optional header")
        directory = opt + 112
        directory_count = uint32(data, opt + 108)
        image_base = struct.unpack_from("<Q", data, opt + 24)[0]
    elif magic == 0x10B:  # PE32
        if optional_size < 96:
            raise InvalidPE("truncated PE32 optional header")
        directory = opt + 96
        directory_count = uint32(data, opt + 92)
        image_base = uint32(data, opt + 28)
    else:
        raise InvalidPE("unsupported optional header")
    # The optional header declares how many data directories exist. Never
    # inspect bytes beyond that count as import metadata: padding or section
    # data could otherwise masquerade as undeclared DLL dependencies.
    if directory_count > (end - directory) // 8:
        raise InvalidPE("data-directory count exceeds optional header")
    sections = []
    for i in range(count):
        offset = end + 40 * i
        if offset + 40 > len(data):
            raise InvalidPE("truncated section table")
        virtual_size, va, raw_size, raw = struct.unpack_from("<IIII", data, offset + 8)
        sections.append((va, virtual_size, raw_size, raw))

    def file_offset(rva: int, size: int = 1) -> int:
        for va, virtual_size, raw_size, raw in sections:
            delta = rva - va
            if delta >= 0 and delta + size <= raw_size and delta < max(virtual_size, raw_size):
                offset = raw + delta
                if offset + size <= len(data):
                    return offset
        raise InvalidPE("import RVA is not backed by file data")

    def name_at(rva: int) -> str:
        offset = file_offset(rva)
        # A DLL name must terminate inside the section's file-backed raw
        # bytes. Otherwise a malformed PE can borrow a NUL from unrelated
        # data beyond the section and produce a misleading dependency list.
        end_bound = min(len(data), offset + 260)
        for va, _virtual_size, raw_size, raw in sections:
            delta = rva - va
            if 0 <= delta < raw_size and raw + delta == offset:
                end_bound = min(end_bound, raw + raw_size)
                break
        end_offset = data.find(b"\0", offset, end_bound)
        if end_offset < 0:
            raise InvalidPE("unterminated DLL name")
        name = data[offset:end_offset].decode("ascii", errors="strict").lower()
        # Import names enter CI JSON/logs. Reject path syntax, whitespace,
        # control characters and log-delimiter characters in untrusted PEs.
        # Windows API-set names and ordinary library basenames use this set.
        if name in {".", ".."} or not re.fullmatch(r"[a-z0-9_.+@-]+", name):
            raise InvalidPE("invalid DLL name")
        return name

    found: set[str] = set()
    for index, record_size, name_pos in ((1, 20, 12), (13, 32, 4)):
        if index >= directory_count:
            continue  # The optional header does not declare this directory.
        entry = directory + 8 * index
        if entry + 8 > end:
            continue  # Older optional headers may omit delay import directory.
        rva = uint32(data, entry)
        size = uint32(data, entry + 4)
        if not rva:
            if size:
                raise InvalidPE("import directory size without RVA")
            continue
        # The data-directory size bounds the descriptor table. Scanning
        # beyond it could turn unrelated section bytes into invented imports
        # or accept a table whose required null terminator is out of bounds.
        if size < record_size:
            raise InvalidPE("import directory too small")
        for number in range(min(1024, size // record_size)):
            offset = file_offset(rva + number * record_size, record_size)
            descriptor = data[offset : offset + record_size]
            if descriptor == bytes(record_size):
                break
            name_rva = struct.unpack_from("<I", descriptor, name_pos)[0]
            if index == 13 and not (uint32(data, offset) & 1):
                # Older delay-load descriptors use virtual addresses.
                name_rva -= image_base
            found.add(name_at(name_rva))
        else:
            raise InvalidPE("unterminated import descriptor table")
    return found


def is_redirected(path: Path) -> bool:
    """Reject symlinks and Windows junction/reparse points without following them."""
    attributes = getattr(path.lstat(), "st_file_attributes", 0)
    return path.is_symlink() or bool(
        attributes & getattr(stat, "FILE_ATTRIBUTE_REPARSE_POINT", 0x400)
    )


def audit(root: Path) -> dict[str, object]:
    # Refuse linked staging roots and linked ancestor paths: otherwise an
    # apparently first-party input may resolve outside the owned checkout.
    # This is a path-layout check, not a TOCTOU or native loader guarantee.
    absolute_root = root.absolute()
    if not absolute_root.is_dir() or any(
        is_redirected(path) for path in (absolute_root, *absolute_root.parents)
    ):
        raise InvalidPE("staged OCR payload path is not a directly owned directory")
    # The runtime resolver rejects extra or redirected content in its
    # app-owned OCR subtree. Apply the same fail-closed layout rule to CI's
    # native inventory, rather than silently ignoring additional payloads.
    # The PE parser also supports synthetic fixtures without tessdata.
    staged_entries = list(root.iterdir())
    if len(staged_entries) > 256:
        raise InvalidPE("staged OCR payload has too many entries")
    # The staging layout must also be unambiguous on a case-insensitive
    # Windows volume, even when this inventory runs on Linux.
    seen_entries: set[str] = set()
    for entry in staged_entries:
        name = entry.name.lower()
        if name in seen_entries:
            raise InvalidPE("case-insensitive runtime entry collision")
        seen_entries.add(name)
        if is_redirected(entry):
            raise InvalidPE("staged OCR payload contains a redirected file")
        if name == "tessdata":
            if not entry.is_dir():
                raise InvalidPE("staged OCR payload has invalid language directory")
            language_entries = list(entry.iterdir())
            # When tessdata is present it must contain exactly one nonempty
            # English model; no other language data or empty placeholder.
            if len(language_entries) != 1:
                raise InvalidPE("staged OCR payload has unreviewed language files")
            language = language_entries[0]
            if (
                language.name.lower() != "eng.traineddata"
                or is_redirected(language)
                or not language.is_file()
                or language.stat().st_size == 0
            ):
                raise InvalidPE("staged OCR payload has invalid English language data")
        elif name == "tesseract.exe" or name.endswith(".dll"):
            if not entry.is_file():
                raise InvalidPE("staged OCR payload has invalid native file")
        else:
            raise InvalidPE("staged OCR payload has unreviewed entries")
    binary = root / "tesseract.exe"
    # Windows resolves DLL names without regard to extension case. Include
    # mixed-case names even when this inventory runs on a Linux host.
    dlls = sorted(
        (p for p in root.iterdir() if p.name.lower().endswith(".dll")),
        key=lambda p: p.name.lower(),
    )
    if not binary.is_file() or not dlls:
        raise InvalidPE("staged OCR executable or runtime DLLs missing")
    files = [binary, *dlls]
    # These basenames appear in artifact JSON and CI logs. Never serialize
    # attacker-controlled path or log delimiters as audited DLL filenames.
    if any(not re.fullmatch(r"[A-Za-z0-9_.+@-]+", file.name) for file in files):
        raise InvalidPE("invalid staged OCR binary name")
    if any(is_redirected(file) for file in files):
        raise InvalidPE("staged OCR payload contains a redirected file")
    names = {f.name.lower() for f in files}
    if len(names) != len(files):
        raise InvalidPE("case-insensitive filename collision")
    records = []
    external: set[str] = set()
    for file in files:
        payload = file.read_bytes()
        dependencies = imports(payload)
        # The Windows release target is x86-64. A mixed-architecture DLL
        # inventory must fail before it can be mistaken for a usable bundle.
        pe = uint32(payload, 0x3C)
        if pe_machine(payload) != 0x8664 or uint16(payload, pe + 24) != 0x20B:
            raise InvalidPE("staged OCR binary is not an AMD64 PE32+ image")
        records.append({
            "binary": file.name,
            "machine": "AMD64",
            "sha256": hashlib.sha256(payload).hexdigest(),
            "size_bytes": len(payload),
            "packaged": sorted(dependencies & names),
            "external_unreviewed": sorted(dependencies - names),
        })
        external.update(dependencies - names)
    # Static import reachability prioritizes the native review queue. It is
    # NOT a removal decision: runtime LoadLibrary calls and optional plugins
    # can require DLLs absent from the PE import graph.
    graph = {
        record["binary"].lower(): set(record["packaged"])
        for record in records
    }
    reachable: set[str] = set()
    pending = ["tesseract.exe"]
    while pending:
        name = pending.pop()
        if name in reachable:
            continue
        reachable.add(name)
        pending.extend(graph[name])
    reachable_external = {
        dependency
        for record in records
        if record["binary"].lower() in reachable
        for dependency in record["external_unreviewed"]
    }
    return {
        "schema_version": 1,
        "qualification": (
            "static PE inventory and import reachability only; dynamic loads, "
            "external DLL resolution and redistribution licenses unreviewed"
        ),
        "binaries": records,
        "statically_reachable_binaries": sorted(reachable),
        "not_statically_reachable_binaries": sorted(names - reachable),
        "external_unreviewed_from_statically_reachable": sorted(reachable_external),
        "external_unreviewed": sorted(external),
    }



def verify_review_register(report: dict[str, object], content: str) -> None:
    """Detect drift from preliminary, non-approved native PE review evidence."""
    rows = {}
    section = ""
    all_external = set()
    reachable_external = set()
    seen_sections = set()
    for line in content.splitlines():
        if line.startswith("## "):
            section = line
        elif line == "All external basenames:":
            section = "all_external"
            seen_sections.add(section)
        elif line == "External basenames from statically reachable binaries:":
            section = "reachable_external"
            seen_sections.add(section)
        if section == "## Packaged binary review register" and line.startswith("| " + chr(96)):
            cells = [cell.strip().strip(chr(96)) for cell in line.strip("|").split("|")]
            if len(cells) != 5:
                raise InvalidPE("malformed native review row")
            name, reachable, size, digest, license_state = cells
            # Windows resolves DLL basenames case-insensitively; real
            # pinned packages include mixed-case names (e.g. libLerc.dll).
            # Normalize identities, while still rejecting case-only duplicates.
            identity = name.lower()
            if (not re.fullmatch(r"(?:tesseract\.exe|[a-z0-9_.+@-]+\.dll)", identity)
                    or identity in rows or reachable not in ("YES", "NO")
                    or not size.isdecimal() or int(size) <= 0
                    or not re.fullmatch(r"[0-9a-f]{64}", digest)
                    or not license_state):
                raise InvalidPE("invalid native review identity")
            rows[identity] = (int(size), digest, reachable == "YES")
        elif section in ("all_external", "reachable_external") and line.startswith("- "):
            match = re.fullmatch(r"- " + chr(96) + r"([a-z0-9_.+@-]+\.dll)" + chr(96), line)
            if match is None:
                raise InvalidPE("invalid external native review row")
            target = all_external if section == "all_external" else reachable_external
            if match.group(1) in target:
                raise InvalidPE("duplicate external native review row")
            target.add(match.group(1))
    if not rows or seen_sections != {"all_external", "reachable_external"}:
        raise InvalidPE("native review register incomplete")
    binaries = report["binaries"]
    observed = {record["binary"].lower(): record for record in binaries}
    if len(observed) != len(binaries) or set(rows) != set(observed):
        raise InvalidPE("staged native binary set differs from review")
    reachable = set(report["statically_reachable_binaries"])
    for name, (size, digest, is_reachable) in rows.items():
        record = observed[name]
        if (record["size_bytes"], record["sha256"]) != (size, digest):
            raise InvalidPE("staged native bytes differ from review")
        if (name in reachable) != is_reachable:
            raise InvalidPE("staged native reachability differs from review")
    if (set(report["external_unreviewed"]) != all_external
            or set(report["external_unreviewed_from_statically_reachable"])
            != reachable_external or not reachable_external.issubset(all_external)):
        raise InvalidPE("staged native external imports differ from review")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("runtime", type=Path, help="staged OCR resource directory")
    parser.add_argument("--output", type=Path, help="write content-free JSON audit artifact")
    parser.add_argument("--review-register", type=Path, help="fail if staged PE identities drift")
    args = parser.parse_args()
    try:
        report = audit(args.runtime)
        if args.review_register is not None:
            verify_review_register(report, args.review_register.read_text(encoding="utf-8"))
        rendered = json.dumps(report, indent=2, sort_keys=True)
        if args.output is not None:
            args.output.write_text(rendered + "\n", encoding="utf-8")
    except (OSError, InvalidPE, UnicodeError, struct.error) as error:
        # Keep unexpected native/OS failures private, but make the fixed
        # preliminary review identity gate actionable without revealing paths.
        print(public_audit_failure(error))
        return 1
    print(rendered)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
