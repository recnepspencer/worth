#!/usr/bin/env python3
"""Every root test that opens a trybuild session runs in nextest's `trybuild-project` group.

trybuild generates one project per crate under `<target>/tests/trybuild` and
builds every crate's cases in one shared target directory there. Within one
test process an in-process mutex serializes sessions; across processes trybuild
relies on a lock file whose heartbeat (`set_len(0)` on an empty file) does not
advance its mtime, so a second process busts the "stale" lock after 1.5 s.
nextest runs each test in its own process, so concurrent sessions overwrite
each other's generated manifest: a should-fail case can be built from another
case's source and pass, or a session can wait out its slow-timeout on the
shared build lock. The `trybuild-project` group runs these tests one at a time;
a queued test does not start its timer until it runs.

This guard finds every `#[test]` function that opens a session, directly or
through worth-proof's compile bundles, and requires the group filter to name
exactly that set. `--print-filter` prints the filter for the config file.
"""

from __future__ import annotations

import pathlib
import re
import sys
import tomllib

ROOT = pathlib.Path(__file__).resolve().parents[2]
CONFIG = ROOT / ".config" / "nextest.toml"
GROUP = "trybuild-project"
SESSION = re.compile(
    r"\bTestCases::new\b|\brun_compile_(?:fail|pass)_(?:bundle|family|families)\s*\("
)
TEST_FN = re.compile(r"#\[test\]\s*(?:#\[[^\]]*\]\s*)*(?:pub(?:\([^)]*\))?\s+)?fn\s+(\w+)\s*\(")


def function_body(text: str, start: int) -> str:
    """The brace-balanced body that follows the signature beginning at `start`."""
    open_at = text.index("{", start)
    depth = 0
    for index in range(open_at, len(text)):
        if text[index] == "{":
            depth += 1
        elif text[index] == "}":
            depth -= 1
            if depth == 0:
                return text[open_at : index + 1]
    raise ValueError(f"unbalanced function body at offset {start}")


ANY_FN = re.compile(r"\bfn\s+(\w+)\s*(?:<[^>{]*>)?\s*\(")
HELPER = re.compile(r"run_compile_(?:fail|pass)_(?:bundle|family|families)")


def session_tests() -> tuple[set[str], list[str]]:
    """Session-opening tests, and any other function that hides a session."""
    names = set()
    hidden = []
    for path in sorted((ROOT / "crates").rglob("*.rs")):
        text = path.read_text(encoding="utf-8")
        if not SESSION.search(text):
            continue
        tests = set()
        for match in TEST_FN.finditer(text):
            tests.add(match.group(1))
            if SESSION.search(function_body(text, match.end())):
                names.add(match.group(1))
        for match in ANY_FN.finditer(text):
            name = match.group(1)
            if name in tests or HELPER.fullmatch(name) or "{" not in text[match.end() :]:
                continue
            if "TestCases::new" in function_body(text, match.end()):
                hidden.append(f"{path.relative_to(ROOT)}: {name}")
    return names, hidden


def configured_tests(profile: str) -> set[str] | None:
    config = tomllib.loads(CONFIG.read_text(encoding="utf-8"))
    overrides = config.get("profile", {}).get(profile, {}).get("overrides", [])
    for override in overrides:
        if override.get("test-group") == GROUP:
            found = re.fullmatch(r"test\(/::\((.*)\)\$/\)", override["filter"])
            return set(found.group(1).split("|")) if found else set()
    return None


def filter_for(names: set[str]) -> str:
    return "test(/::(" + "|".join(sorted(names)) + ")$/)"


def main() -> int:
    expected, hidden = session_tests()
    if "--print-filter" in sys.argv:
        print(filter_for(expected))
        return 0
    groups = tomllib.loads(CONFIG.read_text(encoding="utf-8")).get("test-groups", {})
    failures = [
        f"{site} opens a trybuild session outside a #[test] or compile bundle"
        for site in hidden
    ]
    if groups.get(GROUP, {}).get("max-threads") != 1:
        failures.append(f"[test-groups.{GROUP}] must set max-threads = 1")
    for profile in ("default", "integration", "ci"):
        configured = configured_tests(profile)
        if configured is None:
            failures.append(f"profile.{profile} has no {GROUP} override")
            continue
        for name in sorted(expected - configured):
            failures.append(f"profile.{profile}: trybuild test {name} is outside {GROUP}")
        for name in sorted(configured - expected):
            failures.append(f"profile.{profile}: {name} opens no trybuild session")
    for failure in failures:
        print(failure, file=sys.stderr)
    if not failures:
        print(f"{len(expected)} trybuild tests run in {GROUP}")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
