from __future__ import annotations

import argparse
import csv
import difflib
import json
import os
import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
WORKSPACE = ROOT / "workspaces/worth-ui"
FIXTURE = (
    WORKSPACE
    / "crates/worth-ui-certification/tests/fixtures/compile_contracts/Cargo.toml"
)
INCLUDE = re.compile(r'include!\(\s*"([^"]+)"\s*\)')


@dataclass(frozen=True)
class Case:
    kind: str
    target: str
    source: Path
    snapshot: Path
    owner: str


def execution_inventories() -> tuple[tuple[str, Path], ...]:
    crates = WORKSPACE / "crates"
    return (
        (
            "certification",
            crates
            / "worth-ui-certification/tests/suites/compile_contract_execution.csv",
        ),
        (
            "host",
            crates / "worth-ui-host-contract/tests/suites/compile_contract_cases.csv",
        ),
        (
            "product",
            crates / "worth-ui/tests/suites/compile_contract_execution.csv",
        ),
    )


def fixture_targets() -> dict[Path, str]:
    command = [
        "cargo",
        "metadata",
        "--manifest-path",
        str(FIXTURE),
        "--locked",
        "--no-deps",
        "--format-version",
        "1",
    ]
    metadata = json.loads(subprocess.check_output(command, text=True))
    package = metadata["packages"][0]
    return {
        Path(target["src_path"]).resolve(): target["name"]
        for target in package["targets"]
        if "bin" in target["kind"]
    }


def load_cases() -> list[Case]:
    targets = fixture_targets()
    cases: list[Case] = []
    for owner, inventory in execution_inventories():
        crate_root = inventory.parents[2]
        with inventory.open(newline="", encoding="utf-8") as stream:
            for row in csv.DictReader(stream):
                source = (crate_root / row["path"]).resolve()
                target = targets.pop(source, None)
                if target is None:
                    raise RuntimeError(f"compile fixture target missing for {source}")
                cases.append(
                    Case(
                        kind=row["kind"],
                        target=target,
                        source=source,
                        snapshot=source.with_suffix(".stderr"),
                        owner=owner,
                    )
                )
    if targets:
        extras = ", ".join(sorted(targets.values()))
        raise RuntimeError(f"uninventoried compile fixture targets: {extras}")
    return cases


def cargo_check(cases: list[Case]) -> tuple[int, dict[str, list[dict[str, object]]]]:
    command = [
        "cargo",
        "check",
        "--locked",
        "--keep-going",
        "--manifest-path",
        str(FIXTURE),
        "--message-format=json",
        "--color",
        "never",
    ]
    for case in cases:
        command.extend(("--bin", case.target))
    environment = os.environ.copy()
    environment.setdefault("CARGO_TARGET_DIR", str(WORKSPACE / "target"))
    process = subprocess.run(command, text=True, capture_output=True, env=environment)
    diagnostics: dict[str, list[dict[str, object]]] = {
        case.target: [] for case in cases
    }
    for line in process.stdout.splitlines():
        try:
            message = json.loads(line)
        except json.JSONDecodeError:
            continue
        if message.get("reason") != "compiler-message":
            continue
        target = message.get("target", {}).get("name")
        if target in diagnostics:
            diagnostics[target].append(message["message"])
    return process.returncode, diagnostics


def display_path(path: Path, case: Case) -> str:
    crates = WORKSPACE / "crates"
    if case.owner == "product":
        try:
            return "$WORKSPACE/" + path.relative_to(WORKSPACE).as_posix()
        except ValueError:
            return path.as_posix()
    owner_root = {
        "certification": crates / "worth-ui-certification",
        "host": crates / "worth-ui-host-contract",
    }[case.owner]
    try:
        return path.relative_to(owner_root).as_posix()
    except ValueError:
        try:
            return "$WORKSPACE/" + path.relative_to(WORKSPACE).as_posix()
        except ValueError:
            return path.as_posix()


def source_path(file_name: str) -> Path:
    path = Path(file_name.replace("\\", "/"))
    if not path.is_absolute():
        path = FIXTURE.parent / path
    return path.resolve()


def stable_message(message: str) -> str:
    return re.sub(r"\band \d+ others\b", "and $N others", message)


def is_incidental_private_field_enumeration(message: str) -> bool:
    return "private field" in message and message.endswith("not provided")


def primary_expected_found_evidence(
    primary: dict[str, object] | None, children: list[dict[str, object]]
) -> str | None:
    if primary is None:
        return None
    label = str(primary.get("label") or "")
    if "expected" not in label or "found" not in label:
        return None
    child_evidence = "\n".join(str(child.get("message", "")) for child in children)
    if "expected" in child_evidence and "found" in child_evidence:
        return None
    return label


def canonical_diagnostics(messages: list[dict[str, object]], case: Case) -> str:
    lines: list[str] = []
    for message in messages:
        if message.get("level") != "error":
            continue
        code = message.get("code")
        code_suffix = f"[{code['code']}]" if isinstance(code, dict) else ""
        lines.append(f"error{code_suffix}: {stable_message(str(message['message']))}")
        spans = message.get("spans", [])
        primary = next(
            (span for span in spans if span.get("is_primary")),
            spans[0] if spans else None,
        )
        if primary is not None:
            lines.append(f" --> {display_path(source_path(primary['file_name']), case)}")
        children = message.get("children", [])
        expected_found = primary_expected_found_evidence(primary, children)
        if expected_found is not None:
            lines.append(f" note: {stable_message(expected_found)}")
        for child in children:
            child_message = str(child.get("message", ""))
            if "full name for the type has been written to" in child_message:
                continue
            if child_message == "consider using `--verbose` to print the full type name to the console":
                continue
            if is_incidental_private_field_enumeration(child_message):
                continue
            level = child.get("level", "note")
            lines.append(f" {level}: {stable_message(child_message)}")
        lines.append("")
    return "\n".join(lines).rstrip() + "\n"


def included_sources(source: Path) -> set[Path]:
    covered = {source.resolve()}
    pending = [source.resolve()]
    while pending:
        current = pending.pop()
        for include in INCLUDE.findall(current.read_text(encoding="utf-8")):
            included = (current.parent / include).resolve()
            if included not in covered:
                covered.add(included)
                pending.append(included)
    return covered


def diagnostic_sources(messages: list[dict[str, object]]) -> set[Path]:
    sources: set[Path] = set()
    for message in messages:
        if message.get("level") != "error":
            continue
        for span in message.get("spans", []):
            if span.get("is_primary"):
                sources.add(source_path(span["file_name"]))
    return sources


def assert_failure_snapshots(
    cases: list[Case], diagnostics: dict[str, list[dict[str, object]]], bless: bool
) -> list[str]:
    failures: list[str] = []
    for case in cases:
        messages = diagnostics[case.target]
        errors = [message for message in messages if message.get("level") == "error"]
        if not errors:
            failures.append(f"{case.target}: compiler emitted no error diagnostic")
            continue
        missing_sources = included_sources(case.source) - diagnostic_sources(messages)
        if missing_sources:
            missing = ", ".join(display_path(path, case) for path in sorted(missing_sources))
            failures.append(f"{case.target}: no compiler error from {missing}")
            continue
        actual = canonical_diagnostics(messages, case)
        if bless:
            case.snapshot.write_text(actual, encoding="utf-8", newline="\n")
            continue
        expected = case.snapshot.read_text(encoding="utf-8").replace("\r\n", "\n")
        if actual != expected:
            diff = "".join(
                difflib.unified_diff(
                    expected.splitlines(keepends=True),
                    actual.splitlines(keepends=True),
                    fromfile=str(case.snapshot),
                    tofile=f"{case.target}:actual",
                    n=3,
                )
            )
            failures.append(diff[:8000])
    return failures


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Run Worth UI compiler contracts")
    parser.add_argument(
        "--bless",
        action="store_true",
        help="replace executed compile-fail snapshots with current canonical diagnostics",
    )
    return parser.parse_args()


def main() -> int:
    arguments = parse_args()
    cases = load_cases()
    failing = [case for case in cases if case.kind == "fail"]
    passing = [case for case in cases if case.kind == "pass"]
    fail_status, fail_diagnostics = cargo_check(failing)
    pass_status, pass_diagnostics = cargo_check(passing)

    failures: list[str] = []
    if fail_status == 0:
        failures.append("compile-fail group unexpectedly succeeded")
    failures.extend(
        assert_failure_snapshots(failing, fail_diagnostics, arguments.bless)
    )
    if pass_status != 0:
        failures.append("compile-pass group failed")
        for case in passing:
            for message in pass_diagnostics[case.target]:
                if message.get("level") == "error":
                    failures.append(
                        f"{case.target}: {message.get('rendered') or message.get('message')}"
                    )
    if failures:
        print("\n\n".join(failures), file=sys.stderr)
        return 1
    print(
        f"Worth UI compile contracts passed: {len(failing)} fail targets, "
        f"{len(passing)} pass targets, 2 Cargo sessions"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
