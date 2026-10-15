#!/usr/bin/env python3
"""Every Cargo lockfile resolves a shared dependency line to the same version.

    python scripts/ci/check_lockfile_alignment.py

The root, the Road 1 sub-workspaces, the tools and the certification fixtures
each own a lockfile, and the first-party crates beneath them are compiled by
several of those workspaces. When two lockfiles resolve the same semver line
of a crate to different versions (`generic-array` 0.14.7 here and 0.14.9
there), everything that depends on it, every first-party crate included, gets
a different build identity in each workspace. Shared target directories and
sccache then miss on all of it.

Different semver lines (`syn` 2 next to `syn` 3) are separate crates that no
update can merge, so they are reported by `cargo tree --duplicates`, not here.

To repair a failure, move the lagging lockfile onto the version the others use:

    cargo update --manifest-path <workspace>/Cargo.toml -p <crate> --precise <version>
"""
import sys
import tomllib
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
# Lockfiles that are inputs to tests of the tools themselves, not builds.
FIXTURE_INPUTS = (
    "tools/boundary-check/tests/",
    "crates/worth-proof/tests/",
    "/tests/ui/",
)


def semver_line(version: str) -> str:
    """The range Cargo treats as compatible: major, or minor below 1.0."""
    major, minor, patch = version.split("+")[0].split("-")[0].split(".")[:3]
    if major != "0":
        return major
    if minor != "0":
        return f"0.{minor}"
    return f"0.0.{patch}"


def governed_lockfiles() -> list[Path]:
    return sorted(
        path
        for path in ROOT.rglob("Cargo.lock")
        if "target" not in path.relative_to(ROOT).parts
        and not any(marker in path.as_posix() for marker in FIXTURE_INPUTS)
    )


def resolutions(lockfiles: list[Path]) -> dict[tuple[str, str], dict[str, list[str]]]:
    lines: dict[tuple[str, str], dict[str, list[str]]] = defaultdict(lambda: defaultdict(list))
    for lockfile in lockfiles:
        owner = lockfile.parent.relative_to(ROOT).as_posix() or "."
        for package in tomllib.loads(lockfile.read_text()).get("package", []):
            if "source" not in package:
                continue
            line = (package["name"], semver_line(package["version"]))
            lines[line][package["version"]].append(owner)
    return lines


def main() -> int:
    lockfiles = governed_lockfiles()
    divergent = {
        line: versions for line, versions in resolutions(lockfiles).items() if len(versions) > 1
    }
    if not divergent:
        print(f"[lockfile-alignment] PASS: {len(lockfiles)} lockfiles agree on every shared line")
        return 0
    for (name, line), versions in sorted(divergent.items()):
        print(f"[lockfile-alignment] {name} {line}.x resolves differently:", file=sys.stderr)
        for version, owners in sorted(versions.items()):
            print(f"    {version}: {', '.join(sorted(owners))}", file=sys.stderr)
    return 1


if __name__ == "__main__":
    sys.exit(main())
