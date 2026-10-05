#!/usr/bin/env python3
"""Enforce C.9 runtime-integrity and independent-observer dependency routes,
and keep each limit owner's test-only mint out of every production build."""

from __future__ import annotations

import pathlib
import re
import sys
import tomllib


ROOT = pathlib.Path(__file__).resolve().parents[2]
WORKSPACE = ROOT / "workspaces" / "worth-store"
CRATES = WORKSPACE / "crates"
OBSERVER = CRATES / "worth-store-offline-integrity-observer"
RUNTIME_INTEGRITY = CRATES / "worth-store-physical-integrity"
LOWER_INTEGRITY_DEPENDENCIES = {"worth-foundational", "worth-store-physical-format"}
# Runtime integrity owns budgets (the release-custody head walk) whose refusals
# are sealed `ExhaustedLimit`s; minting one needs the worth-proof law substrate.
# The observer owns no budget and stays on the lower set.
RUNTIME_INTEGRITY_DEPENDENCIES = LOWER_INTEGRITY_DEPENDENCIES | {"worth-proof"}
# Wire encoding/decoding is not a shared physical parser or authority lane.
# Keep this exact allowlist: no runtime codecs, validators, or owner dependencies.
OBSERVER_DEPENDENCIES = LOWER_INTEGRITY_DEPENDENCIES | {"serde", "serde_json"}
# Each owner mints expected limits for other crates' tests behind its
# `test-support` feature. Only dev-dependencies and test-only features may
# enable it: a production edge would let any crate mint an owner's limit.
LIMIT_OWNERS = {
    "worth-store",
    "worth-store-physical-backend",
    "worth-store-physical-integrity",
    "worth-store-recovery-physics",
}
TEST_SUPPORT = "test-support"
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


def production_tables(document: dict) -> list[dict]:
    tables = [document.get(name, {}) for name in ("dependencies", "build-dependencies")]
    for target in document.get("target", {}).values():
        tables.extend(target.get(name, {}) for name in ("dependencies", "build-dependencies"))
    # Workspace-wide features reach every member that inherits the dependency.
    tables.append(document.get("workspace", {}).get("dependencies", {}))
    return tables


def production_test_support(document: dict) -> list[str]:
    """The limit owners whose `test-support` a production edge enables."""
    enabled: list[str] = []
    for table in production_tables(document):
        for name, spec in table.items():
            if not isinstance(spec, dict):
                continue
            owner = spec.get("package", name)
            if owner in LIMIT_OWNERS and TEST_SUPPORT in spec.get("features", []):
                enabled.append(owner)
    # A default feature is on in every production build.
    package = document.get("package", {}).get("name")
    for feature in document.get("features", {}).get("default", []):
        owner, _, enables = feature.rpartition("/")
        owner = owner.rstrip("?") or package
        if enables == TEST_SUPPORT and owner in LIMIT_OWNERS:
            enabled.append(owner)
    return enabled


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
    if runtime_dependencies != RUNTIME_INTEGRITY_DEPENDENCIES:
        violations.append(
            f"{RUNTIME_INTEGRITY.name} production dependencies must be exactly "
            f"{sorted(RUNTIME_INTEGRITY_DEPENDENCIES)}, found {sorted(runtime_dependencies)}"
        )

    with (OBSERVER / "Cargo.toml").open("rb") as source:
        observer_dependencies = all_dependencies(tomllib.load(source))
    if observer_dependencies != OBSERVER_DEPENDENCIES:
        violations.append(
            f"{OBSERVER.name} dependencies of every kind must be exactly "
            f"{sorted(OBSERVER_DEPENDENCIES)}, found {sorted(observer_dependencies)}"
        )

    manifests = [WORKSPACE / "Cargo.toml"]
    manifests.extend(sorted(CRATES.glob("*/Cargo.toml")))
    manifests.extend(sorted((WORKSPACE / "tools").glob("*/Cargo.toml")))
    for manifest in manifests:
        with manifest.open("rb") as source:
            document = tomllib.load(source)
        for owner in production_test_support(document):
            violations.append(
                f"{manifest.relative_to(ROOT)} enables {owner}'s {TEST_SUPPORT} "
                "in a production build"
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

    print(
        "C.9 observer dependency and declaration-only source routes verified; "
        "no production build enables a limit owner's test-support."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
