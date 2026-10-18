"""Decode Cargo test artifacts without coupling build output to gate scheduling."""
import json
from pathlib import Path
import tomllib


def target_harness(configuration, target):
    kinds = target["kind"]
    if any(kind in ("lib", "rlib", "cdylib", "staticlib", "dylib", "proc-macro")
           for kind in kinds):
        return configuration.get("lib", {}).get("harness", True)
    for kind in ("test", "example", "bin", "bench"):
        if kind in kinds:
            for declaration in configuration.get(kind, []):
                if declaration["name"] == target["name"]:
                    return declaration.get("harness", True)
    return True


def test_binaries(stdout):
    binaries, manifests = [], {}
    for line in stdout.splitlines():
        if not line.startswith("{"):
            continue
        record = json.loads(line)
        if record.get("reason") != "compiler-artifact":
            continue
        if not (record.get("executable") and record.get("profile", {}).get("test")):
            continue
        manifest = Path(record["manifest_path"])
        if manifest not in manifests:
            manifests[manifest] = tomllib.loads(manifest.read_text(encoding="utf-8"))
        configuration = manifests[manifest]
        kinds = record["target"]["kind"]
        binaries.append({
            "package": configuration["package"]["name"], "target": record["target"]["name"],
            "lib": any(k in ("lib", "rlib", "cdylib", "proc-macro") for k in kinds),
            "harness": target_harness(configuration, record["target"]),
            "exe": record["executable"], "cwd": str(manifest.parent),
        })
    return binaries
