#!/usr/bin/env python3
"""Deterministic court/testkit source, target and resolved-graph inventory.

Read saved `cargo metadata --format-version 1` JSON. No builds or timing
thresholds. Direct normal/dev package IDs define a court's dependency surface.
"""
import argparse
from collections import defaultdict
import json
import os
from pathlib import Path


def rust_lines(directory):
    if os.name == "nt" and not str(directory).startswith("\\\\?\\"):
        directory = Path("\\\\?\\" + str(directory.resolve()))
    total = 0
    for parent, children, files in os.walk(directory):
        parent = Path(parent)
        children[:] = [child for child in children if child not in ("target", ".git")
                       and not (parent / child / "Cargo.toml").exists()]
        total += sum(len((parent / name).read_text(encoding="utf-8").splitlines())
                     for name in files if name.endswith(".rs"))
    return total


def measure(metadata):
    members = set(metadata["workspace_members"])
    nodes = {node["id"]: node for node in metadata["resolve"]["nodes"]}
    packages = {package["id"]: package for package in metadata["packages"]}
    rows, signatures = [], defaultdict(list)
    for identity in sorted(members):
        package = packages[identity]
        name = package["name"]
        role = "court" if "-court-" in name else "testkit" if "-testkit" in name else "product"
        edges = [{"package": dep["pkg"], "alias": dep["name"],
                  "kinds": sorted({kind["kind"] or "normal" for kind in dep["dep_kinds"]})}
                 for dep in nodes[identity]["deps"]]
        direct = sorted({edge["package"] for edge in edges
                         if set(edge["kinds"]) & {"normal", "dev"}})
        targets = [{"name": target["name"], "kind": target["kind"],
                    "test": target["test"], "doctest": target["doctest"]}
                   for target in package["targets"]]
        row = {"package": name, "id": identity, "role": role,
               "rust_lines": rust_lines(Path(package["manifest_path"]).parent),
               "targets": targets, "edges": edges, "normal_dev_surface": direct}
        rows.append(row)
        if role == "court":
            signatures[tuple(direct)].append(row)
    equal = []
    for surface, group in signatures.items():
        if len(group) > 1:
            equal.append({"courts": sorted(row["package"] for row in group),
                          "combined_lines": sum(row["rust_lines"] for row in group),
                          "surface": list(surface)})
    return {"workspace": metadata["workspace_root"], "packages": rows,
            "test_targets": sum(target["test"] for row in rows for target in row["targets"]),
            "integration_targets": sum("test" in target["kind"] for row in rows for target in row["targets"]),
            "court_crates": sum(row["role"] == "court" for row in rows),
            "testkit_crates": sum(row["role"] == "testkit" for row in rows),
            "equal_dependency_sets": equal,
            "over_ceiling": [row["package"] for row in rows
                             if row["role"] != "product" and row["rust_lines"] > 15_000]}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("metadata", type=Path, nargs="+")
    parser.add_argument("--out", required=True, type=Path)
    args = parser.parse_args()
    reports = [measure(json.loads(path.read_text(encoding="utf-8"))) for path in args.metadata]
    args.out.write_text(json.dumps(reports, indent=2) + "\n", encoding="utf-8")
    for report in reports:
        print(report["workspace"], {key: report[key] for key in
              ("test_targets", "integration_targets", "court_crates", "testkit_crates", "over_ceiling")})
        for duplicate in report["equal_dependency_sets"]:
            print("equal surface:", duplicate["courts"], "combined lines:", duplicate["combined_lines"])
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
