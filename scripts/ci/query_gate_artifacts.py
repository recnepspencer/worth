"""Decode Cargo test artifacts without coupling build output to gate scheduling."""
import json
import re
from pathlib import Path


def package_name(manifest):
    found = re.search(r'(?m)^name\s*=\s*"([^"]+)"', manifest.read_text(encoding="utf-8"))
    return found.group(1) if found else manifest.parent.name


def test_binaries(stdout):
    binaries = []
    for line in stdout.splitlines():
        if not line.startswith("{"):
            continue
        record = json.loads(line)
        if record.get("reason") != "compiler-artifact":
            continue
        if not (record.get("executable") and record.get("profile", {}).get("test")):
            continue
        manifest = Path(record["manifest_path"])
        kinds = record["target"]["kind"]
        binaries.append({
            "package": package_name(manifest), "target": record["target"]["name"],
            "lib": any(k in ("lib", "rlib", "cdylib", "proc-macro") for k in kinds),
            "exe": record["executable"], "cwd": str(manifest.parent),
        })
    return binaries
