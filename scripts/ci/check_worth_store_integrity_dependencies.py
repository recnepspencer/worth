#!/usr/bin/env python3
"""Enforce C.9 runtime-integrity and independent-observer dependency routes."""

from __future__ import annotations

import pathlib
import re
import sys
import tomllib


ROOT = pathlib.Path(__file__).resolve().parents[2]
CRATES = ROOT / "workspaces" / "worth-store" / "crates"
OBSERVER = CRATES / "worth-store-offline-integrity-observer"
RUNTIME_INTEGRITY = CRATES / "worth-store-physical-integrity"
LOWER_INTEGRITY_DEPENDENCIES = {"worth-foundational", "worth-store-physical-format"}
# Wire encoding/decoding is not a shared physical parser or authority lane.
# Keep this exact allowlist: no runtime codecs, validators, or owner dependencies.
OBSERVER_DEPENDENCIES = LOWER_INTEGRITY_DEPENDENCIES | {"serde", "serde_json"}
FORMAT_CRATE = re.compile(r"\bworth_store_physical_format\b")
DECLARATION_ROUTE = re.compile(r"\s*::\s*integrity_declarations\b")


def production_dependencies(document: dict) -> set[str]:
    dependencies: set[str] = set()
    for table_name in ("dependencies", "build-dependencies"):
        dependencies.update(document.get(table_name, {}))
    for target in document.get("target", {}).values():
        for table_name in ("dependencies", "build-dependencies"):
            dependencies.update(target.get(table_name, {}))
    return dependencies


def all_dependencies(document: dict) -> set[str]:
    dependencies = production_dependencies(document)
    dependencies.update(document.get("dev-dependencies", {}))
    for target in document.get("target", {}).values():
        dependencies.update(target.get("dev-dependencies", {}))
    return dependencies


def forbidden_format_routes(source: str) -> list[int]:
    lines: list[int] = []
    for match in FORMAT_CRATE.finditer(source):
        if not DECLARATION_ROUTE.match(source, match.end()):
            lines.append(source.count("\n", 0, match.start()) + 1)
    return lines


def main() -> int:
    violations: list[str] = []
    with (RUNTIME_INTEGRITY / "Cargo.toml").open("rb") as source:
        runtime_dependencies = production_dependencies(tomllib.load(source))
    if runtime_dependencies != LOWER_INTEGRITY_DEPENDENCIES:
        violations.append(
            f"{RUNTIME_INTEGRITY.name} production dependencies must be exactly "
            f"{sorted(LOWER_INTEGRITY_DEPENDENCIES)}, found {sorted(runtime_dependencies)}"
        )

    with (OBSERVER / "Cargo.toml").open("rb") as source:
        observer_dependencies = all_dependencies(tomllib.load(source))
    if observer_dependencies != OBSERVER_DEPENDENCIES:
        violations.append(
            f"{OBSERVER.name} dependencies of every kind must be exactly "
            f"{sorted(OBSERVER_DEPENDENCIES)}, found {sorted(observer_dependencies)}"
        )

    for source_root in (OBSERVER / "src", OBSERVER / "tests"):
        if not source_root.exists():
            continue
        for path in sorted(source_root.rglob("*.rs")):
            source = path.read_text(encoding="utf-8")
            for line in forbidden_format_routes(source):
                relative = path.relative_to(ROOT)
                violations.append(
                    f"{relative}:{line} reaches physical-format outside "
                    "integrity_declarations"
                )

    if violations:
        print("C.9 integrity dependency violations:", file=sys.stderr)
        for violation in violations:
            print(f"- {violation}", file=sys.stderr)
        return 1

    print("C.9 observer dependency and declaration-only source routes verified.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
