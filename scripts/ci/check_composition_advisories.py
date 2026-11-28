#!/usr/bin/env python3
"""Composition guards for WORTH Rust sources.

Single source of truth for three checks, called from CI, the git pre-commit
hook, agent edit-time hooks, and `cargo test`. Keep the logic here; every other
surface is a shim.

  HARD      file exceeds 400 lines            -> exit 1 (allowlistable)
  ADVISORY  function body exceeds 60 lines    -> reported
  ADVISORY  function takes 5+ parameters      -> reported

Advisories do not fail by default; pass --advisories-fatal to gate them.

`clippy.toml` raises `too-many-arguments-threshold` to 32 and justifies it with
"repository-specific function scrutiny ... remain the composition enforcement".
This script is that scrutiny. Before it existed, the justification pointed at
nothing.

Scope defaults to `dirty` — a change is judged on what it touches. The
repository has known pre-existing over-cap files being reduced deliberately;
they must never block unrelated work. `workspace` scope is a full audit, not a
gate.

Usage:
  check_composition_advisories.py [dirty|changed|workspace] [--json]
                                  [--advisories-fatal] [--files A.rs B.rs ...]
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from dataclasses import dataclass, field
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]

FILE_LINE_CAP = 400
FUNCTION_LINE_ADVISORY = 60
PARAMETER_ADVISORY = 5

FILE_CAP_ALLOWLIST = ROOT / "scripts/ci/workspace_rust_line_cap_allowlist.txt"

WORKSPACE_GLOBS = (
    "crates/**/*.rs",
    "workspaces/worth-query-bank-world/crates/**/*.rs",
    "workspaces/worth-query/crates/**/*.rs",
    "workspaces/worth-store/crates/**/*.rs",
    "workspaces/worth-store/tools/**/*.rs",
)

FN_PATTERN = re.compile(r"\bfn\s+(?P<name>[A-Za-z_][A-Za-z0-9_]*)")


@dataclass
class Findings:
    hard: list[dict] = field(default_factory=list)
    advisory: list[dict] = field(default_factory=list)


def load_allowlist() -> set[str]:
    if not FILE_CAP_ALLOWLIST.is_file():
        return set()
    return {
        line.strip()
        for line in FILE_CAP_ALLOWLIST.read_text(encoding="utf-8").splitlines()
        if line.strip() and not line.startswith("#")
    }


def git_lines(args: list[str]) -> list[str]:
    result = subprocess.run(
        ["git", *args], cwd=ROOT, capture_output=True, text=True, check=False
    )
    if result.returncode != 0:
        return []
    return [line for line in result.stdout.split("\0") if line]


def working_tree_changes() -> set[str]:
    names: set[str] = set()
    names.update(git_lines(["diff", "--name-only", "--diff-filter=ACMR", "-z", "--", "*.rs"]))
    names.update(
        git_lines(["diff", "--cached", "--name-only", "--diff-filter=ACMR", "-z", "--", "*.rs"])
    )
    names.update(git_lines(["ls-files", "--others", "--exclude-standard", "-z", "--", "*.rs"]))
    return names


def merge_base_changes() -> set[str]:
    """Files this branch changed relative to its base.

    `dirty` means nothing on a clean CI checkout, which would leave the guard
    silently vacuous exactly where it is meant to gate. This resolves a base
    ref instead, so a pull request is still judged on what it changed.
    """
    import os

    candidates = []
    if base := os.environ.get("GITHUB_BASE_REF"):
        candidates.append(f"origin/{base}")
    candidates += ["origin/master", "origin/main", "master", "main"]

    for ref in candidates:
        if not git_lines(["rev-parse", "--verify", "--quiet", f"{ref}^{{commit}}"]) and (
            subprocess.run(
                ["git", "rev-parse", "--verify", "--quiet", f"{ref}^{{commit}}"],
                cwd=ROOT,
                capture_output=True,
                check=False,
            ).returncode
            != 0
        ):
            continue
        merge_base = subprocess.run(
            ["git", "merge-base", "HEAD", ref], cwd=ROOT, capture_output=True, text=True, check=False
        )
        if merge_base.returncode != 0:
            continue
        base_sha = merge_base.stdout.strip()
        return set(
            git_lines(
                ["diff", "--name-only", "--diff-filter=ACMR", "-z", base_sha, "--", "*.rs"]
            )
        )
    return set()


def collect_files(scope: str, explicit: list[str] | None) -> list[Path]:
    if explicit:
        return [ROOT / f for f in explicit if f.endswith(".rs")]
    if scope == "workspace":
        names = set(git_lines(["ls-files", "-z", "--", *WORKSPACE_GLOBS]))
    elif scope == "changed":
        names = working_tree_changes() | merge_base_changes()
    else:  # dirty
        names = working_tree_changes()
    return sorted({ROOT / n for n in names})


def strip_noise(source: str) -> str:
    """Blank out comments and string/char literal bodies, preserving offsets.

    Brace and paren matching must not be thrown off by a `{` inside a comment
    or a string. Newlines are preserved so line numbers stay exact.
    """
    out = list(source)
    i, n = 0, len(source)
    while i < n:
        ch = source[i]
        if ch == "/" and i + 1 < n and source[i + 1] == "/":
            while i < n and source[i] != "\n":
                out[i] = " "
                i += 1
        elif ch == "/" and i + 1 < n and source[i + 1] == "*":
            depth = 1
            out[i] = out[i + 1] = " "
            i += 2
            while i < n and depth:
                if source.startswith("/*", i):
                    depth += 1
                    out[i] = out[i + 1] = " "
                    i += 2
                elif source.startswith("*/", i):
                    depth -= 1
                    out[i] = out[i + 1] = " "
                    i += 2
                else:
                    if source[i] != "\n":
                        out[i] = " "
                    i += 1
        elif ch == '"':
            # Raw strings: r"..", r#".."#
            hashes = 0
            j = i - 1
            while j >= 0 and source[j] == "#":
                hashes += 1
                j -= 1
            is_raw = j >= 0 and source[j] == "r"
            out[i] = " "
            i += 1
            if is_raw:
                terminator = '"' + "#" * hashes
                end = source.find(terminator, i)
                end = n if end == -1 else end
                while i < end:
                    if source[i] != "\n":
                        out[i] = " "
                    i += 1
            else:
                while i < n:
                    if source[i] == "\\":
                        out[i] = " "
                        if i + 1 < n and source[i + 1] != "\n":
                            out[i + 1] = " "
                        i += 2
                        continue
                    if source[i] == '"':
                        out[i] = " "
                        i += 1
                        break
                    if source[i] != "\n":
                        out[i] = " "
                    i += 1
        else:
            i += 1
    return "".join(out)


def match_group(text: str, start: int, opener: str, closer: str) -> int:
    """Index just past the group opened at `start`, or -1."""
    depth = 0
    i = start
    while i < len(text):
        if text[i] == opener:
            depth += 1
        elif text[i] == closer:
            depth -= 1
            if depth == 0:
                return i + 1
        i += 1
    return -1


def count_parameters(params: str) -> int:
    """Top-level comma count, excluding a leading `self` receiver."""
    depth_paren = depth_angle = depth_bracket = 0
    parts, current = [], []
    i = 0
    while i < len(params):
        ch = params[i]
        if ch == "-" and i + 1 < len(params) and params[i + 1] == ">":
            current.append("->")
            i += 2
            continue
        if ch in "([":
            depth_paren += ch == "("
            depth_bracket += ch == "["
        elif ch in ")]":
            depth_paren -= ch == ")"
            depth_bracket -= ch == "]"
        elif ch == "<":
            depth_angle += 1
        elif ch == ">":
            depth_angle = max(0, depth_angle - 1)
        elif ch == "," and depth_paren == depth_angle == depth_bracket == 0:
            parts.append("".join(current))
            current = []
            i += 1
            continue
        current.append(ch)
        i += 1
    parts.append("".join(current))
    cleaned = [p.strip() for p in parts if p.strip()]
    if cleaned and re.match(r"^(&\s*('\w+\s*)?(mut\s+)?)?(self|mut\s+self)\b", cleaned[0]):
        cleaned = cleaned[1:]
    return len(cleaned)


def analyse(path: Path, findings: Findings, allowlist: set[str]) -> None:
    try:
        source = path.read_text(encoding="utf-8")
    except (OSError, UnicodeDecodeError):
        return
    rel = path.relative_to(ROOT).as_posix()

    total_lines = source.count("\n") + (0 if source.endswith("\n") or not source else 1)
    if total_lines > FILE_LINE_CAP and rel not in allowlist:
        findings.hard.append(
            {
                "kind": "file-line-cap",
                "file": rel,
                "line": 1,
                "actual": total_lines,
                "limit": FILE_LINE_CAP,
                "message": f"{rel} is {total_lines} lines (cap {FILE_LINE_CAP})",
            }
        )

    text = strip_noise(source)
    for match in FN_PATTERN.finditer(text):
        name = match.group("name")
        cursor = match.end()
        # Optional generics before the parameter list.
        while cursor < len(text) and text[cursor].isspace():
            cursor += 1
        if cursor < len(text) and text[cursor] == "<":
            end_generics = match_group(text, cursor, "<", ">")
            if end_generics == -1:
                continue
            cursor = end_generics
        while cursor < len(text) and text[cursor].isspace():
            cursor += 1
        if cursor >= len(text) or text[cursor] != "(":
            continue
        params_end = match_group(text, cursor, "(", ")")
        if params_end == -1:
            continue

        line_no = text.count("\n", 0, match.start()) + 1
        param_count = count_parameters(text[cursor + 1 : params_end - 1])
        if param_count >= PARAMETER_ADVISORY:
            findings.advisory.append(
                {
                    "kind": "function-parameters",
                    "file": rel,
                    "line": line_no,
                    "function": name,
                    "actual": param_count,
                    "limit": PARAMETER_ADVISORY,
                    "message": f"{rel}:{line_no} fn {name} takes {param_count} parameters "
                    f"(advisory {PARAMETER_ADVISORY})",
                }
            )

        body_start = text.find("{", params_end)
        semicolon = text.find(";", params_end)
        if body_start == -1 or (semicolon != -1 and semicolon < body_start):
            continue  # trait method declaration, no body
        body_end = match_group(text, body_start, "{", "}")
        if body_end == -1:
            continue
        body_lines = text.count("\n", body_start, body_end)
        if body_lines > FUNCTION_LINE_ADVISORY:
            findings.advisory.append(
                {
                    "kind": "function-length",
                    "file": rel,
                    "line": line_no,
                    "function": name,
                    "actual": body_lines,
                    "limit": FUNCTION_LINE_ADVISORY,
                    "message": f"{rel}:{line_no} fn {name} body is {body_lines} lines "
                    f"(advisory {FUNCTION_LINE_ADVISORY})",
                }
            )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "scope",
        nargs="?",
        default="dirty",
        choices=["dirty", "changed", "workspace"],
        help="dirty (default): working-tree changes. changed: this branch vs its "
        "merge-base, for CI. workspace: everything, for a full audit only — the "
        "repository has known pre-existing over-cap files that are being chipped "
        "away deliberately and must never block a change.",
    )
    parser.add_argument("--files", nargs="*", help="explicit paths, overrides scope")
    parser.add_argument("--json", action="store_true", help="machine-readable output")
    parser.add_argument(
        "--advisories-fatal",
        action="store_true",
        help="exit non-zero on advisories as well as hard violations",
    )
    args = parser.parse_args()

    files = collect_files(args.scope, args.files)
    findings = Findings()
    allowlist = load_allowlist()
    for path in files:
        if path.is_file():
            analyse(path, findings, allowlist)

    if args.json:
        print(
            json.dumps(
                {
                    "scanned": len(files),
                    "hard": findings.hard,
                    "advisory": findings.advisory,
                },
                indent=2,
            )
        )
    else:
        label = args.scope if not args.files else "explicit"
        print(f"[composition-guards] scanned {len(files)} Rust files ({label})")
        for item in findings.hard:
            print(f"FAIL: {item['message']}")
        for item in sorted(findings.advisory, key=lambda i: -i["actual"]):
            print(f"ADVISORY: {item['message']}")
        if findings.hard:
            print(f"[composition-guards] {len(findings.hard)} hard violation(s)")
        if findings.advisory:
            print(f"[composition-guards] {len(findings.advisory)} advisory finding(s)")
        if not findings.hard and not findings.advisory:
            print("[composition-guards] PASS")

    if findings.hard:
        return 1
    if args.advisories_fatal and findings.advisory:
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
