#!/usr/bin/env python3
"""Frame budget lane: no owned function's static frame exceeds the ceiling.

`run` builds the roster declared in `frame_budget.json` with the committed
profiles and measures exactly the executables that build reported. Each
executable must match its PDB (RSDS GUID and age). The lane reads x64 PE unwind
metadata (`.pdata` and `UNWIND_INFO`), attributes each function's fixed stack
allocation to its symbol through the PDB, and refuses when a function owned by
a declared crate prefix exceeds `ceiling_bytes`. A future held by value occupies
its holder's frame, so async state machines are covered; total stack depth and
heap-held futures are not. The lane is Windows-only, because the binding stack
is the 1 MiB Windows main thread; elsewhere it refuses instead of passing.

Usage:
  python scripts/ci/frame_budget.py run [--report PATH]
  python scripts/ci/frame_budget.py check --binary LABEL=PATH ... [--report PATH]
"""

from __future__ import annotations

import argparse
import bisect
import dataclasses
import json
import os
import re
import struct
import subprocess
import sys
import uuid
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
CONFIG = Path(__file__).resolve().with_name("frame_budget.json")

UNW_FLAG_CHAININFO = 4
MSF_MAGIC = b"Microsoft C/C++ MSF 7.00\r\n\x1aDS\0\0\0"


class UnwindError(ValueError):
    """The binary's metadata is not a supported, consistent x64 layout."""


def unwind_frame_bytes(image: bytes, offset_of, rva: int, seen: tuple[int, ...] = ()) -> int:
    """Bytes the prolog allocates below the return address, chains included.

    Pushed registers, fixed allocations and machine frames count; the return
    address does not. `offset_of` maps an RVA to a file offset in `image`.
    """
    if rva in seen:
        raise UnwindError(f"cyclic unwind chain at {rva:#x}")
    p = offset_of(rva)
    version_flags, _prolog, count, _frame = image[p : p + 4]
    if version_flags & 7 not in (1, 2):
        raise UnwindError(f"unsupported UNWIND_INFO version at {rva:#x}")
    flags = version_flags >> 3
    total = 0
    slot = 0
    while slot < count:
        code = image[p + 4 + slot * 2 + 1]
        operation, info = code & 15, code >> 4
        slot += 1
        if operation == 0:  # UWOP_PUSH_NONVOL
            total += 8
        elif operation == 1:  # UWOP_ALLOC_LARGE
            if info == 0:
                total += struct.unpack_from("<H", image, p + 4 + slot * 2)[0] * 8
                slot += 1
            elif info == 1:
                total += struct.unpack_from("<I", image, p + 4 + slot * 2)[0]
                slot += 2
            else:
                raise UnwindError(f"bad UWOP_ALLOC_LARGE at {rva:#x}")
        elif operation == 2:  # UWOP_ALLOC_SMALL
            total += info * 8 + 8
        elif operation in (3, 6):  # UWOP_SET_FPREG, UWOP_EPILOG (v2)
            pass
        elif operation in (4, 8):  # UWOP_SAVE_NONVOL, UWOP_SAVE_XMM128
            slot += 1
        elif operation in (5, 9):  # the _FAR forms
            slot += 2
        elif operation == 10:  # UWOP_PUSH_MACHFRAME
            total += 40 + 8 * info
        else:
            raise UnwindError(f"unsupported unwind operation {operation} at {rva:#x}")
    if flags & UNW_FLAG_CHAININFO:
        chain = p + 4 + ((count + 1) // 2) * 4
        parent = struct.unpack_from("<I", image, chain + 8)[0]
        total += unwind_frame_bytes(image, offset_of, parent, (*seen, rva))
    return total


def primary_start(image: bytes, offset_of, unwind_rva: int, begin: int) -> int:
    """The start of the function a chained (cold) fragment belongs to."""
    p = offset_of(unwind_rva)
    version_flags, _, count, _ = image[p : p + 4]
    if (version_flags >> 3) & UNW_FLAG_CHAININFO:
        chain = p + 4 + ((count + 1) // 2) * 4
        parent_begin, _, parent_unwind = struct.unpack_from("<III", image, chain)
        return primary_start(image, offset_of, parent_unwind, parent_begin)
    return begin


@dataclasses.dataclass
class PeImage:
    data: bytes
    base: int
    sections: list[tuple[int, int, int, int]]
    functions: list[tuple[int, int, int]]

    @classmethod
    def read(cls, path: Path) -> "PeImage":
        return cls.parse(path.read_bytes(), path.name)

    @classmethod
    def parse(cls, data: bytes, name: str = "image") -> "PeImage":
        pe = struct.unpack_from("<I", data, 0x3C)[0]
        if data[pe : pe + 4] != b"PE\0\0":
            raise UnwindError(f"{name} is not a PE image")
        count = struct.unpack_from("<H", data, pe + 6)[0]
        optional = pe + 24
        if struct.unpack_from("<H", data, optional)[0] != 0x20B:
            raise UnwindError(f"{name} is not x64 PE32+")
        base = struct.unpack_from("<Q", data, optional + 24)[0]
        first = optional + struct.unpack_from("<H", data, pe + 20)[0]
        sections = [struct.unpack_from("<IIII", data, first + 40 * i + 8) for i in range(count)]
        image = cls(data, base, sections, [])
        rva, size = image.directory(3)
        start = image.offset(rva) if size else 0
        image.functions = [struct.unpack_from("<III", data, start + i) for i in range(0, size, 12)]
        return image

    def directory(self, index: int) -> tuple[int, int]:
        pe = struct.unpack_from("<I", self.data, 0x3C)[0]
        return struct.unpack_from("<II", self.data, pe + 24 + 112 + index * 8)

    def offset(self, rva: int) -> int:
        for virtual_size, virtual, raw_size, raw in self.sections:
            if virtual <= rva < virtual + max(virtual_size, raw_size):
                return raw + rva - virtual
        raise UnwindError(f"RVA {rva:#x} outside every section")

    def codeview(self) -> tuple[bytes, int]:
        """The (GUID, age) this executable records for its PDB."""
        rva, size = self.directory(6)
        if not size:
            raise UnwindError("executable has no debug directory")
        start = self.offset(rva)
        for entry in range(start, start + size, 28):
            kind, _length, _address, pointer = struct.unpack_from("<IIII", self.data, entry + 12)
            if kind == 2 and self.data[pointer : pointer + 4] == b"RSDS":
                guid = self.data[pointer + 4 : pointer + 20]
                return guid, struct.unpack_from("<I", self.data, pointer + 20)[0]
        raise UnwindError("executable records no CodeView PDB identity")

    def pdb_name(self) -> str:
        """The PDB file name the linker recorded (cargo underscores hyphens)."""
        rva, size = self.directory(6)
        start = self.offset(rva) if size else 0
        for entry in range(start, start + size, 28):
            kind, _length, _address, pointer = struct.unpack_from("<IIII", self.data, entry + 12)
            if kind == 2 and self.data[pointer : pointer + 4] == b"RSDS":
                end = self.data.index(b"\0", pointer + 24)
                return re.split(r"[\\/]", self.data[pointer + 24 : end].decode(errors="replace"))[-1]
        raise UnwindError("executable records no CodeView PDB identity")


class MsfFile:
    """Streams of an MSF 7.00 (PDB) container held in `data` (bytes or mmap)."""

    def __init__(self, data) -> None:
        if data[: len(MSF_MAGIC)] != MSF_MAGIC:
            raise UnwindError("not an MSF 7.00 PDB")
        self.data = data
        self.block, _free, _count, directory_bytes, _unknown, map_block = struct.unpack_from("<6I", data, 32)
        directory_blocks = -(-directory_bytes // self.block)
        map_indices = struct.unpack_from(f"<{directory_blocks}I", data, map_block * self.block)
        directory = self.gather(map_indices, directory_bytes)
        streams = struct.unpack_from("<I", directory, 0)[0]
        self.sizes = struct.unpack_from(f"<{streams}I", directory, 4)
        cursor = 4 + 4 * streams
        self.blocks: list[tuple[int, ...]] = []
        for length in self.sizes:
            count = 0 if length == 0xFFFFFFFF else -(-length // self.block)
            self.blocks.append(struct.unpack_from(f"<{count}I", directory, cursor))
            cursor += 4 * count

    def gather(self, indices, length: int) -> bytes:
        block = self.block
        return b"".join(self.data[index * block : (index + 1) * block] for index in indices)[:length]

    def stream(self, index: int) -> bytes:
        if index >= len(self.sizes) or self.sizes[index] == 0xFFFFFFFF:
            raise UnwindError(f"PDB has no stream {index}")
        return self.gather(self.blocks[index], self.sizes[index])

    def identity(self) -> tuple[bytes, int]:
        """The (GUID, age) in the PDB information stream."""
        info = self.stream(1)
        return info[12:28], struct.unpack_from("<I", info, 8)[0]

    def module_ranges(self) -> list[tuple[int, int, int, str]]:
        """(section, offset, size, module object name) for every code contribution."""
        dbi = self.stream(3)
        module_bytes, contribution_bytes = struct.unpack_from("<ii", dbi, 24)
        names: list[str] = []
        cursor = 64
        end = 64 + module_bytes
        while cursor < end:
            cursor += 64  # Unused1, SectionContr, flags, streams, sizes, files, offsets
            module_end = dbi.index(b"\0", cursor)
            names.append(dbi[cursor:module_end].decode(errors="replace"))
            object_end = dbi.index(b"\0", module_end + 1)
            cursor = (object_end + 1 + 3) & ~3
        start = end
        version = struct.unpack_from("<I", dbi, start)[0]
        width = 32 if version == 0xEFFE0000 + 20140516 else 28
        ranges = []
        for entry in range(start + 4, start + contribution_bytes, width):
            section, _, offset, size, characteristics, module = struct.unpack_from("<HHiiIH", dbi, entry)
            if characteristics & 0x20 and module < len(names):  # IMAGE_SCN_CNT_CODE
                ranges.append((section, offset, size, names[module]))
        return ranges


def pdb_identity(data) -> tuple[bytes, int]:
    """The (GUID, age) in an MSF 7.00 PDB's information stream."""
    return MsfFile(data).identity()


def open_pdb(path: Path):
    """A read-only map of `path`; PDBs of court binaries reach gigabytes."""
    import mmap

    with path.open("rb") as handle:
        return mmap.mmap(handle.fileno(), 0, access=mmap.ACCESS_READ)


def require_matching_pdb(path: Path, image: PeImage) -> MsfFile:
    """Refuse an executable whose PDB came from another build."""
    pdb = path.parent / image.pdb_name()
    if not pdb.is_file():
        raise UnwindError(f"{path.name} has no PDB {pdb.name} beside it")
    msf = MsfFile(open_pdb(pdb))
    if msf.identity() != image.codeview():
        raise UnwindError(f"{pdb.name} does not belong to {path.name}; rebuild both")
    return msf


def module_crate(module: str) -> str:
    """The crate a rustc object module was compiled for (`<crate>-<hash>.<cgu>.rcgu.o`)."""
    stem = re.split(r"[\\/]", module)[-1]
    return re.split(r"[-.]", stem, maxsplit=1)[0]


def contributing_crate(image: PeImage, ranges, rva: int) -> str | None:
    """The crate whose object contributed the code at `rva`."""
    for section, offset, size, module in ranges:
        if not 1 <= section <= len(image.sections):
            continue
        start = image.sections[section - 1][1] + offset
        if start <= rva < start + size:
            return module_crate(module)
    return None


def strip_generics(name: str) -> str:
    """Remove generic arguments; keep path components, impl and closure names."""
    out: list[str] = []
    depth = 0
    for index, char in enumerate(name):
        arrow = index > 0 and name[index - 1] == "-"
        if char == "<" and not arrow:
            depth += 1
            continue
        if char == ">" and depth and not arrow:
            depth -= 1
            continue
        if depth == 0:
            out.append(char)
    return "".join(out)


def owner_path(symbol: str) -> str:
    """The defining path of a demangled symbol, without instantiation details."""
    value = symbol.strip()
    if value.startswith("<") and " as " in value:
        # `<Type as Trait>::method` is defined beside the impl's self type.
        value = value[1:].split(" as ", 1)[0] + "::" + value.rsplit(">::", 1)[-1]
    value = strip_generics(value)
    value = re.sub(r"::impl\$\d+", "", value)
    value = re.sub(r"::h[0-9a-f]{16}$", "", value)
    return value.strip(":")


def owned(path: str, prefixes: list[str]) -> bool:
    crate = path.split("::", 1)[0]
    return any(crate.startswith(prefix) for prefix in prefixes)


def pdb_symbols(path: Path, base: int) -> dict[int, list[str]]:
    """RVA to symbol names, from the binary's PDB through DbgHelp."""
    import ctypes as c
    import ctypes.wintypes as w

    kernel = c.WinDLL("kernel32")
    help_ = c.WinDLL("dbghelp")
    kernel.GetCurrentProcess.restype = c.c_void_p
    help_.SymInitializeW.argtypes = [c.c_void_p, w.LPCWSTR, w.BOOL]
    help_.SymLoadModuleExW.argtypes = [
        c.c_void_p, c.c_void_p, w.LPCWSTR, w.LPCWSTR, c.c_ulonglong, w.DWORD, c.c_void_p, w.DWORD,
    ]
    help_.SymLoadModuleExW.restype = c.c_ulonglong
    help_.SymCleanup.argtypes = [c.c_void_p]
    callback = c.WINFUNCTYPE(c.c_int, c.c_void_p, c.c_uint, c.c_void_p)
    help_.SymEnumSymbols.argtypes = [c.c_void_p, c.c_ulonglong, c.c_char_p, callback, c.c_void_p]
    process = kernel.GetCurrentProcess()
    help_.SymSetOptions(0x10 | 0x200)  # SYMOPT_LOAD_LINES off; FAIL_CRITICAL_ERRORS
    if not help_.SymInitializeW(process, str(path.parent), False):
        raise UnwindError("SymInitializeW failed")
    names: dict[int, list[str]] = {}
    try:
        loaded = help_.SymLoadModuleExW(process, None, str(path), None, base, 0, None, 0)
        if not loaded:
            raise UnwindError(f"no symbols for {path}")

        @callback
        def collect(symbol, _size, _context):
            # SYMBOL_INFO: Address at 56, NameLen at 76, Name at 84.
            address = c.c_ulonglong.from_address(symbol + 56).value - loaded
            length = c.c_uint.from_address(symbol + 76).value
            raw = c.string_at(symbol + 84, length).decode(errors="replace")
            names.setdefault(address, [])
            if raw not in names[address]:
                names[address].append(raw)
            return 1

        if not help_.SymEnumSymbols(process, loaded, b"*", collect, None):
            raise UnwindError(f"cannot enumerate symbols of {path}")
    finally:
        help_.SymCleanup(process)
    return names


@dataclasses.dataclass
class Frame:
    path: str
    bytes: int
    binary: str


def measure(label: str, path: Path, floor: int, prefixes: list[str]) -> tuple[list[Frame], list[dict]]:
    """Owned frames at or above `floor`, and large entries no symbol names."""
    image = PeImage.read(path)
    msf = require_matching_pdb(path, image)
    large: dict[int, int] = {}
    for begin, _end, unwind in image.functions:
        size = unwind_frame_bytes(image.data, image.offset, unwind)
        if size >= floor:
            start = primary_start(image.data, image.offset, unwind, begin)
            large[start] = max(large.get(start, 0), size)
    names = pdb_symbols(path, image.base)
    addresses = sorted(names)
    frames: list[Frame] = []
    unnamed: list[dict] = []
    ranges = None
    for start, size in large.items():
        symbols = names.get(start)
        if not symbols:
            # Crates built without debug information name only public symbols.
            # The contributing object module still identifies the crate.
            ranges = msf.module_ranges() if ranges is None else ranges
            crate = contributing_crate(image, ranges, start)
            if crate is None or owned(crate, prefixes):
                # The nearest preceding symbol only helps a reader find the code.
                before = addresses[bisect.bisect_left(addresses, start) - 1] if addresses else None
                nearest = names[before][0] if before is not None and before < start else None
                unnamed.append({"binary": label, "rva": start, "bytes": size, "crate": crate, "after_symbol": nearest})
            continue
        for defining in {owner_path(symbol) for symbol in symbols}:
            if owned(defining, prefixes):
                frames.append(Frame(defining, size, label))
    return frames, unnamed


def deduplicate(frames: list[Frame]) -> list[dict]:
    by_path: dict[str, dict] = {}
    for frame in frames:
        row = by_path.setdefault(frame.path, {"path": frame.path, "bytes": 0, "binaries": []})
        row["bytes"] = max(row["bytes"], frame.bytes)
        if frame.binary not in row["binaries"]:
            row["binaries"].append(frame.binary)
    return sorted(by_path.values(), key=lambda row: (-row["bytes"], row["path"]))


def built_executables(stdout: str, wanted: list[str]) -> dict[str, Path]:
    """Executables a cargo JSON message stream reported for the wanted targets."""
    found: dict[str, Path] = {}
    for line in stdout.splitlines():
        if not line.startswith("{"):
            continue
        message = json.loads(line)
        name = message.get("target", {}).get("name")
        if message.get("reason") == "compiler-artifact" and message.get("executable") and name in wanted:
            found[name] = Path(message["executable"])
    return found


def build_roster(config: dict) -> list[tuple[str, Path]]:
    """Build each roster entry and keep exactly the executables it reported."""
    roster = []
    environment = {**os.environ, "WORTH_IMPLEMENTATION_BUILD_ID": str(uuid.uuid4())}
    for entry in config["roster"]:
        command = ["cargo", *entry["cargo"], "--message-format=json-render-diagnostics"]
        print("build", entry["label"], " ".join(command), flush=True)
        built = subprocess.run(command, cwd=REPO_ROOT, env=environment, stdout=subprocess.PIPE, text=True)
        if built.returncode:
            raise SystemExit(f"frame budget: build of {entry['label']} failed")
        found = built_executables(built.stdout, entry["executables"])
        missing = sorted(set(entry["executables"]) - set(found))
        if missing:
            raise SystemExit(f"frame budget: {entry['label']} built no executable for {missing}")
        roster += [(f"{entry['label']}:{name}", path) for name, path in sorted(found.items())]
    return roster


def explicit_roster(items: list[str]) -> list[tuple[str, Path]]:
    pairs = [item.split("=", 1) for item in items]
    return [(label, Path(path)) for label, path in pairs]


def bands(rows: list[dict]) -> dict[str, int]:
    return {
        ">=256KiB": sum(row["bytes"] >= 262144 for row in rows),
        "128-256KiB": sum(131072 <= row["bytes"] < 262144 for row in rows),
        "64-128KiB": sum(65536 <= row["bytes"] < 131072 for row in rows),
        "32-64KiB": sum(32768 <= row["bytes"] < 65536 for row in rows),
    }


def measure_roster(arguments: argparse.Namespace) -> int:
    if sys.platform != "win32":
        print("frame budget: unsupported platform; the lane measures Windows x64 PE binaries")
        return 2
    config = json.loads(CONFIG.read_text(encoding="utf-8"))
    ceiling = config["ceiling_bytes"]
    prefixes = config["owner_crate_prefixes"]
    frames: list[Frame] = []
    unnamed: list[dict] = []
    roster = build_roster(config) if arguments.command == "run" else explicit_roster(arguments.binary)
    for label, path in roster:
        found, missing = measure(label, path, config["census_floor_bytes"], prefixes)
        frames += found
        unnamed += missing
        print(f"{label}: {len(found)} owned frames >= {config['census_floor_bytes']} bytes", flush=True)
    rows = deduplicate(frames)
    over = [row for row in rows if row["bytes"] > ceiling]
    unnamed_over = [row for row in unnamed if row["bytes"] > ceiling]
    report = {
        "ceiling_bytes": ceiling,
        "binaries": {label: str(path) for label, path in roster},
        "bands": bands(rows),
        "over_ceiling": over,
        "unnamed_over_ceiling": unnamed_over,
        "census": rows,
    }
    if arguments.report:
        Path(arguments.report).write_text(json.dumps(report, indent=2), encoding="utf-8")
    print("bands", report["bands"])
    for row in over:
        print(f"OVER {row['bytes']} {row['path']} [{', '.join(row['binaries'])}]")
    for row in unnamed_over:
        print(f"UNNAMED OVER {row['bytes']} rva {row['rva']:#x} crate {row['crate']} in {row['binary']}")
    if over or unnamed_over:
        print(f"frame budget: {len(over) + len(unnamed_over)} functions exceed {ceiling} bytes")
        return 1
    print(f"frame budget: every owned frame is within {ceiling} bytes")
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    commands = parser.add_subparsers(dest="command", required=True)
    run = commands.add_parser("run", help="build the declared roster and measure it")
    run.add_argument("--report", help="write the census JSON here")
    check = commands.add_parser("check", help="measure given executables (diagnostic)")
    check.add_argument("--binary", action="append", required=True, help="LABEL=PATH")
    check.add_argument("--report", help="write the census JSON here")
    return measure_roster(parser.parse_args(argv))


if __name__ == "__main__":
    sys.exit(main())
