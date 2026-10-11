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
import struct
from pathlib import Path


class InvalidPE(ValueError):
    """Malformed or unsupported PE import metadata."""


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
    magic = uint16(data, opt)
    if magic == 0x20B:  # PE32+
        directory = opt + 112
        image_base = struct.unpack_from("<Q", data, opt + 24)[0]
    elif magic == 0x10B:  # PE32
        directory = opt + 96
        image_base = uint32(data, opt + 28)
    else:
        raise InvalidPE("unsupported optional header")
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
        end_offset = data.find(b"\0", offset, min(len(data), offset + 260))
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
        entry = directory + 8 * index
        if entry + 8 > end:
            continue  # Older optional headers may omit delay import directory.
        rva = uint32(data, entry)
        if not rva:
            continue
        for number in range(1024):
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


def audit(root: Path) -> dict[str, object]:
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
    if root.is_symlink() or any(file.is_symlink() for file in files):
        raise InvalidPE("staged OCR payload contains a symbolic link")
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


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("runtime", type=Path, help="staged OCR resource directory")
    parser.add_argument("--output", type=Path, help="write content-free JSON audit artifact")
    args = parser.parse_args()
    try:
        report = audit(args.runtime)
        rendered = json.dumps(report, indent=2, sort_keys=True)
        if args.output is not None:
            args.output.write_text(rendered + "\n", encoding="utf-8")
    except (OSError, InvalidPE, UnicodeError, struct.error):
        # Fail without emitting install paths or raw system error strings.
        print("Windows PE dependency audit failed: malformed or inaccessible staged payload.")
        return 1
    print(rendered)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
