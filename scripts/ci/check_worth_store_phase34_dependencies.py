#!/usr/bin/env python3
"""Enforce Worth Store's physical-owner -> layout-adapter -> courtroom direction."""

from __future__ import annotations

import pathlib
import re
import sys
import tomllib


ROOT = pathlib.Path(__file__).resolve().parents[2]
STORE = ROOT / "workspaces" / "worth-store"
CRATES = STORE / "crates"

PHYSICAL_OWNERS = {
    "worth-store-lsm-authority",
    "worth-store-physical-backend",
    "worth-store-physical-format",
    "worth-store-physical-integrity",
    "worth-store-physical-isolation",
    "worth-store-recovery-physics",
    "worth-store-wal",
}
UPWARD_CRATES = {
    "worth-store-certification",
    "worth-store-layout-indexes",
    "worth-store-test-support",
}
LAYOUT_LOWER_OWNERS = {
    "worth-store-blob-chunks",
    "worth-store-tiering",
}
LAYOUT_OBSERVATION_CONSUMERS = {
    "worth-store-maintenance": {"observation"},
    # S.10: layout indexes "own rebuild, quarantine, and repair behavior for
    # their artifact families" and operations owns "canonical owner-plan DAG
    # scheduling", so operations schedules the layout repair consequence owner.
    "worth-store-operations": {
        "access_planning",
        "bootstrap",
        "declarations",
        "integrity",
        "materialization",
        "observation",
        "operational_repair",
    },
}
LAYOUT_PACKAGE = "worth-store-layout-indexes"
IDENTIFIER = re.compile(r"[A-Za-z_][A-Za-z0-9_]*|\*")
NON_CODE = re.compile(
    r"""//[^\n]*"""  # line and doc comments
    r"""|/\*.*?\*/"""  # block comments (non-nested)
    r"""|\bb?r(#*)".*?"\1"""  # raw strings
    r"""|(?:\bb)?"(?:\\.|[^"\\])*\""""  # strings
    r"""|(?:\bb)?'(?:\\.|[^'\\])'""",  # char literals, not lifetimes
    re.DOTALL,
)
LAYOUT_HARNESS_ROOTS = (
    CRATES / "worth-store-test-support" / "src" / "harness" / "layout",
    CRATES / "worth-store-test-support" / "src" / "harness" / "lsm_execution_fixture",
)
FORBIDDEN_LAYOUT_HARNESS_AUTHORITY = (
    "BaselineBTreeExactCounterWitness::",
    "AdmittedPhysicalReadRequest {",
    "AdmittedPhysicalRecoveryRequest {",
    "AdmittedPhysicalMutationRequest {",
    "from_planned_counter_envelope(",
    "select_with_budget(",
)


def manifest(path: pathlib.Path) -> dict:
    with path.open("rb") as source:
        return tomllib.load(source)


def production_dependencies(document: dict) -> set[str]:
    names: set[str] = set()
    for table_name in ("dependencies", "build-dependencies"):
        names.update(document.get(table_name, {}))
    for target in document.get("target", {}).values():
        for table_name in ("dependencies", "build-dependencies"):
            names.update(target.get(table_name, {}))
    return names


def layout_crate_names(document: dict) -> set[str]:
    """Every Rust name under which a manifest can reach the layout crate,
    including a `package = ...` rename in any dependency table."""
    tables = [document.get(name, {}) for name in ("dependencies", "dev-dependencies")]
    for target in document.get("target", {}).values():
        tables.extend(target.get(name, {}) for name in ("dependencies", "dev-dependencies"))
    names: set[str] = set()
    for table in tables:
        for key, spec in table.items():
            package = spec.get("package", key) if isinstance(spec, dict) else key
            if package == LAYOUT_PACKAGE:
                names.add(key.replace("-", "_"))
    return names


def mask_non_code(source: str) -> str:
    """Blank comments and literals, keeping offsets and newlines, so only code
    paths are read and every reported line number stays exact."""
    return NON_CODE.sub(lambda match: re.sub(r"[^\n]", " ", match.group(0)), source)


def skip_whitespace(source: str, index: int) -> int:
    while index < len(source) and source[index].isspace():
        index += 1
    return index


def use_tree_roots(source: str, index: int):
    """Yield (offset, first segment) for each leaf of the use tree at `index`,
    descending through grouped, nested and multi-line `{...}` trees."""
    index = skip_whitespace(source, index)
    if index >= len(source):
        return
    if source[index] != "{":
        segment = IDENTIFIER.match(source, index)
        yield index, segment.group(0) if segment else source[index]
        return
    index += 1
    while True:
        index = skip_whitespace(source, index)
        if index >= len(source) or source[index] == "}":
            return
        yield from use_tree_roots(source, index)
        depth = 0
        while index < len(source):
            character = source[index]
            if character == "{":
                depth += 1
            elif character == "}":
                if depth == 0:
                    return
                depth -= 1
            elif character == "," and depth == 0:
                index += 1
                break
            index += 1


def layout_module_imports(source: str, crate_names: set[str]):
    """Yield (offset, module) for every path into the layout crate; a root
    alias, glob, `self` or bare crate name yields the crate root itself."""
    masked = mask_non_code(source)
    for name in sorted(crate_names):
        for match in re.finditer(rf"\b{name}\b", masked):
            index = skip_whitespace(masked, match.end())
            if masked.startswith("::", index):
                yield from use_tree_roots(masked, index + 2)
            else:
                yield match.start(), f"{name} (the crate root)"


def layout_import_violations(
    source: str, crate_names: set[str], allowed_modules: set[str]
) -> list[tuple[int, str]]:
    return [
        (source.count("\n", 0, offset) + 1, module)
        for offset, module in layout_module_imports(source, crate_names)
        if module not in allowed_modules
    ]


def self_test() -> list[str]:
    """The import guard must see every spelling a consumer could use."""
    names = {"worth_store_layout_indexes"}
    allowed = {"observation", "declarations", "access_planning"}
    cases = {
        "use worth_store_layout_indexes::{\n    access_planning,\n    artifact_family::X,\n};": [
            (3, "artifact_family")
        ],
        "use worth_store_layout_indexes::{observation::{A, B}, {keyspace::K}};": [
            (1, "keyspace")
        ],
        "use worth_store_layout_indexes::{self, observation::A};": [(1, "self")],
        "use worth_store_layout_indexes::*;": [(1, "*")],
        "use worth_store_layout_indexes as layout;": [
            (1, "worth_store_layout_indexes (the crate root)")
        ],
        "fn f() { worth_store_layout_indexes::keyspace::run() }": [(1, "keyspace")],
        "use worth_store_layout_indexes::{observation::A, declarations::{B, C}};": [],
        "// worth_store_layout_indexes::keyspace\nlet s = \"worth_store_layout_indexes::keyspace\";": [],
    }
    failures = []
    for source, expected in cases.items():
        found = layout_import_violations(source, names, allowed)
        if found != expected:
            failures.append(f"self-test: {source!r} found {found}, expected {expected}")
    renamed = {"dependencies": {"layout": {"package": LAYOUT_PACKAGE, "path": "x"}}}
    if layout_crate_names(renamed) != {"layout"}:
        failures.append("self-test: a renamed layout dependency escapes the import guard")
    return failures


def main() -> int:
    violations: list[str] = self_test()
    crate_dependencies: dict[str, set[str]] = {}

    for path in sorted(CRATES.glob("*/Cargo.toml")):
        document = manifest(path)
        name = document.get("package", {}).get("name")
        if name:
            crate_dependencies[name] = production_dependencies(document)

    for owner in sorted(PHYSICAL_OWNERS):
        forbidden = crate_dependencies.get(owner, set()) & UPWARD_CRATES
        for dependency in sorted(forbidden):
            violations.append(f"{owner} must not depend on {dependency}")

    layout_forbidden = crate_dependencies.get("worth-store-layout-indexes", set()) & {
        "worth-store-certification",
        "worth-store-test-support",
    }
    for dependency in sorted(layout_forbidden):
        violations.append(f"worth-store-layout-indexes must not depend on {dependency}")

    for owner in sorted(LAYOUT_LOWER_OWNERS):
        if "worth-store-layout-indexes" in crate_dependencies.get(owner, set()):
            violations.append(f"{owner} must not depend upward on worth-store-layout-indexes")

    for crate, allowed_modules in LAYOUT_OBSERVATION_CONSUMERS.items():
        crate_names = layout_crate_names(manifest(CRATES / crate / "Cargo.toml"))
        crate_names.add(LAYOUT_PACKAGE.replace("-", "_"))
        source_paths = [
            path
            for directory in ("src", "tests", "benches", "examples")
            for path in sorted((CRATES / crate / directory).rglob("*.rs"))
        ]
        for path in source_paths:
            source = path.read_text(encoding="utf-8")
            for line_number, imported in layout_import_violations(
                source, crate_names, allowed_modules
            ):
                relative = path.relative_to(STORE)
                violations.append(
                    f"{relative}:{line_number} imports layout owner module {imported}; "
                    f"consumer crates may import only {sorted(allowed_modules)}"
                )

            if "layout_projection" in path.parts and path.name != "tests.rs":
                for forbidden in ("fn admit_", "fn readmit_", "fn execute_", "fn issue_"):
                    if forbidden in source:
                        relative = path.relative_to(STORE)
                        violations.append(
                            f"{relative} exposes authority-shaped operation {forbidden.strip()} "
                            "from an observation projection module"
                        )

    for root in LAYOUT_HARNESS_ROOTS:
        if not root.exists():
            # A moved harness must move this guard with it, never silently escape it.
            violations.append(f"layout harness guard root {root.relative_to(STORE)} is missing")
            continue
        paths = root.rglob("*.rs") if root.is_dir() else (root,)
        for path in paths:
            source = path.read_text(encoding="utf-8")
            for forbidden in FORBIDDEN_LAYOUT_HARNESS_AUTHORITY:
                if forbidden in source:
                    relative = path.relative_to(STORE)
                    violations.append(
                        f"{relative} constructs displaced layout authority through {forbidden}"
                    )

    if violations:
        print("Phase 34 dependency-direction violations:", file=sys.stderr)
        for violation in violations:
            print(f"- {violation}", file=sys.stderr)
        return 1

    print(f"Phase 34 dependency direction verified across {len(crate_dependencies)} crates.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
