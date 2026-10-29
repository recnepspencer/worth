#!/usr/bin/env python3
"""Keep compile-boundary sessions in nextest's process-serial group.

The compile_fail module path owns membership. Renaming a test cannot remove
its trybuild serialization. query_gate imports session_tests() for the same
membership while it executes test binaries directly.
"""

from __future__ import annotations

import pathlib
import re
import sys
import tomllib

ROOT = pathlib.Path(__file__).resolve().parents[2]
CONFIG = ROOT / ".config" / "nextest.toml"
GROUP = "trybuild-project"
FILTER = "test(/^compile_fail::/)"
SESSION = re.compile(
    r"\bTestCases::new\b|\brun_compile_(?:fail|pass)_(?:bundle|family|families)\s*\("
)
TEST_FN = re.compile(
    r"#\[test\]\s*(?:#\[[^\]]*\]\s*)*(?:pub(?:\([^)]*\))?\s+)?fn\s+(\w+)\s*\("
)


def compile_boundary_path(path: pathlib.Path, crate: pathlib.Path) -> bool:
    relative = path.relative_to(crate / "tests")
    return (
        len(relative.parts) >= 2
        and relative.parts[0] == crate.name.replace("-", "_")
        and relative.parts[1] in ("compile_fail", "compile_fail.rs")
    )


def session_tests() -> tuple[set[str], list[str]]:
    """Compile-boundary test names and misplaced session owners.

    Keep the leaf-name set used by query_gate. Membership comes from the cost
    path, including sessions invoked through an owning compile bundle.
    """
    names: set[str] = set()
    misplaced: list[str] = []
    for crate in sorted((ROOT / "crates").iterdir()):
        tests = crate / "tests"
        if not tests.is_dir():
            continue
        for path in sorted(tests.rglob("*.rs")):
            text = path.read_text(encoding="utf-8")
            functions = set(TEST_FN.findall(text))
            if compile_boundary_path(path, crate):
                names.update(functions)
            elif functions and SESSION.search(text):
                misplaced.append(str(path.relative_to(ROOT)))
        # Unit tests are fast by definition. A compiler session left in src
        # would escape the compile_fail path group during nextest execution.
        for path in sorted((crate / "src").rglob("*.rs")):
            text = path.read_text(encoding="utf-8")
            if SESSION.search(text):
                misplaced.append(str(path.relative_to(ROOT)))
    return names, misplaced


def main() -> int:
    if "--print-filter" in sys.argv:
        print(FILTER)
        return 0
    expected, misplaced = session_tests()
    config = tomllib.loads(CONFIG.read_text(encoding="utf-8"))
    failures = [f"{site}: compiler session outside compile_fail::" for site in misplaced]
    if config.get("test-groups", {}).get(GROUP, {}).get("max-threads") != 1:
        failures.append(f"[test-groups.{GROUP}] must set max-threads = 1")
    default_overrides = config.get("profile", {}).get("default", {}).get("overrides", [])
    for profile in ("default", "integration", "ci"):
        overrides = config.get("profile", {}).get(profile, {}).get("overrides", [])
        # Custom profiles inherit default overrides in nextest.
        effective = overrides if profile == "default" else overrides + default_overrides
        filters = [o["filter"] for o in effective if o.get("test-group") == GROUP]
        if filters != [FILTER]:
            failures.append(f"profile.{profile}: {GROUP} must select {FILTER} exactly once")
    for failure in failures:
        print(failure, file=sys.stderr)
    if not failures:
        print(f"{len(expected)} compile-boundary tests run in {GROUP} by module path")
    return int(bool(failures))


if __name__ == "__main__":
    sys.exit(main())
