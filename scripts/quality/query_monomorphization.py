"""Measure a Windows Rust archive using LLVM's independent object reports."""
import argparse
from collections import Counter, defaultdict
import hashlib
import json
from itertools import zip_longest
from pathlib import Path
import re
import subprocess
from tempfile import TemporaryFile


def lines(command):
    # Diagnostics cannot fill a second pipe while a large archive streams out.
    with TemporaryFile(mode="w+t", encoding="utf-8") as error:
        with subprocess.Popen(command, stdout=subprocess.PIPE, stderr=error,
                              text=True, encoding="utf-8", errors="strict") as process:
            for line in process.stdout:
                yield line.rstrip("\r\n")
            if process.wait() != 0:
                error.seek(0)
                raise RuntimeError(f"{command!r} failed: {error.read()}")


def family(name):
    if name.startswith("<"):
        depth = 0
        separator = None
        for end, char in enumerate(name):
            if depth == 1 and name.startswith(" as ", end):
                separator = end
            depth += (char == "<") - (char == ">" and name[end - 1] != "-")
            if depth == 0:
                owner_end = separator if separator is not None else end
                name = family(name[1:owner_end]) + name[end + 1:]
                break
    result = []
    depth = 0
    for index, char in enumerate(name):
        if char == "<":
            if depth == 0:
                result.append("<>")
            depth += 1
        elif char == ">" and (index == 0 or name[index - 1] != "-"):
            depth -= 1
        elif depth == 0:
            result.append(char)
    return "".join(result)


def measure(archive, llvm):
    nm = [str(llvm / "llvm-nm.exe"), "--defined-only", "--extern-only", "--no-sort",
          "--format=just-symbols", str(archive)]
    raw_names = (line for line in lines(nm) if line and not line.endswith(":"))
    demangled_names = (line for line in lines(nm + ["--demangle"])
                       if line and not line.endswith(":"))
    names = {}
    nm_count = 0
    for raw, demangled in zip_longest(raw_names, demangled_names):
        if raw is None or demangled is None:
            raise RuntimeError("Raw and demangled symbol reports disagree")
        names[raw] = demangled
        nm_count += 1
    owners, modules, sizes, copies = Counter(), Counter(), Counter(), Counter()
    code_names = set()
    shared_sizes, shared_copies = Counter(), Counter()
    code_bytes = attributed_bytes = symbol_count = objects = 0
    sections, symbols = {}, []

    def finish_object():
        nonlocal code_bytes, attributed_bytes, symbol_count, objects
        if not sections:
            return
        objects += 1
        by_section = defaultdict(list)
        for symbol in symbols:
            section = symbol.get("section", 0)
            if not symbol.get("external") or section <= 0:
                continue
            if symbol["name"] not in names:
                raise RuntimeError("Defined external symbol is missing from the independent nm report")
            name = names[symbol["name"]]
            stem = family(name)
            parts = stem.split("::")
            owners[parts[0]] += 1
            modules["::".join(parts[:3])] += 1
            symbol_count += 1
            if symbol.get("function"):
                by_section[section].append(dict(symbol, name=name))
        for number, section in sections.items():
            if not section.get("code"):
                continue
            code_names.add(section["name"])
            size = section["size"]
            code_bytes += size
            positions = defaultdict(list)
            for symbol in by_section[number]:
                positions[symbol["value"]].append(symbol["name"])
            offsets = sorted(positions)
            for index, offset in enumerate(offsets):
                end = offsets[index + 1] if index + 1 < len(offsets) else size
                if offset < 0 or offset > end or end > size:
                    raise RuntimeError("Function coordinate exceeds its COFF section")
                # Alias names at one address share one physical contribution.
                stems = tuple(sorted({family(name) for name in positions[offset]}))
                if len(stems) == 1:
                    sizes[stems[0]] += end - offset
                    copies[stems[0]] += 1
                else:
                    shared_sizes[stems] += end - offset
                    shared_copies[stems] += 1
                attributed_bytes += end - offset

    item = None
    kind = None
    for line in lines([str(llvm / "llvm-readobj.exe"), "--sections", "--symbols",
                       str(archive)]):
        if line.startswith("File: "):
            finish_object()
            sections, symbols = {}, []
        elif line.startswith("Format: ") and not line.startswith("Format: COFF"):
            raise RuntimeError(f"Expected COFF, got {line}")
        elif line in ("  Section {", "  Symbol {"):
            item, kind = {}, line.strip().split()[0]
        elif line == "  }" and item is not None:
            if kind == "Section":
                sections[item["number"]] = item
            else:
                symbols.append(item)
            item = kind = None
        elif item is not None:
            value = line.strip()
            if value.startswith("Name: "):
                item["name"] = value[6:].split(" (", 1)[0] if kind == "Section" else value[6:]
            elif kind == "Section":
                if value.startswith("Number: "):
                    item["number"] = int(value[8:])
                elif value.startswith("RawDataSize: "):
                    item["size"] = int(value[13:])
                elif value.startswith("IMAGE_SCN_CNT_CODE"):
                    item["code"] = True
            elif value.startswith("Section: "):
                item["section"] = int(re.search(r"\((-?\d+)\)$", value)[1])
            elif value.startswith("Value: "):
                item["value"] = int(value[7:])
            elif value.startswith("StorageClass: "):
                item["external"] = value == "StorageClass: External (0x2)"
            elif value.startswith("ComplexType: "):
                item["function"] = value == "ComplexType: Function (0x2)"
    finish_object()

    size_total = 0
    for line in lines([str(llvm / "llvm-size.exe"), "-A", str(archive)]):
        fields = line.split()
        if len(fields) == 3 and fields[0] in code_names:
            size_total += int(fields[1])
    if nm_count != symbol_count or size_total != code_bytes:
        raise RuntimeError(f"Independent LLVM reports disagree: symbols {symbol_count}/{nm_count}, "
                           f"code bytes {code_bytes}/{size_total}")
    if not symbol_count or not attributed_bytes:
        raise RuntimeError("Archive produced no defined external symbols or attributed code")
    with archive.open("rb") as source:
        archive_hash = hashlib.file_digest(source, "sha256").hexdigest()
    return {
        "archive": str(archive), "rlib_bytes": archive.stat().st_size, "objects": objects,
        "archive_sha256": archive_hash,
        "exported_symbols": symbol_count, "code_bytes": code_bytes,
        "unattributed_code_bytes": code_bytes - attributed_bytes,
        "shared_alias_code_bytes": sum(shared_sizes.values()),
        "shared_alias_groups": [{"names": list(names), "bytes": size,
                                 "copies": shared_copies[names]}
                                for names, size in shared_sizes.most_common()],
        "independent_nm_symbols": nm_count, "independent_size_code_bytes": size_total,
        "owners": dict(owners.most_common()), "modules": dict(modules.most_common()),
        "function_families": [{"name": name, "bytes": size, "copies": copies[name]}
                              for name, size in sizes.most_common()],
    }


def cargo_archive(messages, package):
    """Select an actual successful Cargo build output, never a stale glob match."""
    data = messages.read_bytes()
    encoding = "utf-16" if data.startswith((b"\xff\xfe", b"\xfe\xff")) else "utf-8-sig"
    archives = set()
    finished = False
    for line in data.decode(encoding).splitlines():
        if not line.startswith("{"):
            continue
        message = json.loads(line)
        if message.get("reason") == "build-finished":
            finished = message["success"]
        elif message.get("reason") == "compiler-artifact":
            if message["target"]["name"] == package.replace("-", "_"):
                archives.update(Path(name) for name in message["filenames"] if name.endswith(".rlib"))
    if not finished or len(archives) != 1:
        raise RuntimeError("Cargo did not report one requested archive from a successful build")
    return archives.pop()


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    inputs = parser.add_mutually_exclusive_group(required=True)
    inputs.add_argument("--archive", type=Path)
    inputs.add_argument("--cargo-messages", type=Path)
    parser.add_argument("--package", default="worth-query-testkit-certification-canonical")
    parser.add_argument("--llvm-bin", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    archive = args.archive if args.archive else cargo_archive(args.cargo_messages, args.package)
    result = measure(archive.resolve(strict=True), args.llvm_bin.resolve(strict=True))
    args.output.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({key: result[key] for key in (
        "rlib_bytes", "exported_symbols", "code_bytes", "unattributed_code_bytes")}))
