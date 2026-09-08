#!/usr/bin/env python3
"""Build the independent observer, then run the C.9 root process courtroom."""

from __future__ import annotations

import json
import os
import pathlib
import subprocess
import sys


ROOT = pathlib.Path(__file__).resolve().parents[2]
MANIFEST = ROOT / "workspaces" / "worth-store" / "Cargo.toml"
OBSERVER_PACKAGE = "worth-store-offline-integrity-observer"
OBSERVER_BINARY = "physical_store_integrity_observer"
CERTIFICATION_PACKAGE = "worth-store-physical-certification"
OBSERVER_ENV = "WORTH_C9_OBSERVER_EXECUTABLE"


def build_executable(package: str, target_name: str, target_args: list[str]) -> pathlib.Path:
    command = [
        "cargo",
        *target_args,
        "--manifest-path",
        str(MANIFEST),
        "-p",
        package,
        "--message-format=json-render-diagnostics",
    ]
    result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True)
    executable: pathlib.Path | None = None
    for line in result.stdout.splitlines():
        try:
            message = json.loads(line)
        except json.JSONDecodeError:
            continue
        target = message.get("target", {})
        if (
            message.get("reason") == "compiler-artifact"
            and target.get("name") == target_name
            and message.get("executable")
        ):
            executable = pathlib.Path(message["executable"])
    if result.returncode != 0:
        sys.stdout.write(result.stdout)
        sys.stderr.write(result.stderr)
        raise SystemExit(result.returncode)
    if executable is None or not executable.is_file():
        raise SystemExit(f"Cargo did not report the freshly built {target_name} executable")
    return executable.resolve()


def run_courtroom(observer: pathlib.Path, certification: pathlib.Path) -> int:
    environment = os.environ.copy()
    environment[OBSERVER_ENV] = str(observer)
    owner_tests = [
        str(certification),
        "c9_integrity_localization",
    ]
    owner_result = subprocess.run(owner_tests, cwd=ROOT, env=environment)
    if owner_result.returncode != 0:
        return owner_result.returncode
    family_graph = owner_tests[:-1] + [
        "c9_integrity_localization::offline_process::independent_observer_traverses_current_family_graph",
        "--exact",
        "--ignored",
        "--nocapture",
    ]
    family_result = subprocess.run(family_graph, cwd=ROOT, env=environment)
    if family_result.returncode != 0:
        return family_result.returncode
    courtroom = [
        str(certification),
        "c9_integrity_localization::c9_root_protocol_process_courtroom",
        "--exact",
        "--ignored",
        "--nocapture",
    ]
    courtroom_result = subprocess.run(
        courtroom, cwd=ROOT, env=environment, capture_output=True, text=True
    )
    sys.stdout.write(courtroom_result.stdout)
    sys.stderr.write(courtroom_result.stderr)
    if courtroom_result.returncode != 0:
        return courtroom_result.returncode
    # Child process output can appear between libtest's name and its final `ok`.
    # Require the selected outer harness summary rather than adjacent fragments.
    summaries = [line for line in courtroom_result.stdout.splitlines()
                 if line.startswith("test result:")]
    if not summaries or not summaries[-1].startswith("test result: ok. 1 passed; 0 failed;"):
        sys.stderr.write(
            "C.9 courtroom command exited successfully without executing the exact test\n"
        )
        return 1
    return 0


def main() -> int:
    observer = build_executable(OBSERVER_PACKAGE, OBSERVER_BINARY,
                                ["build", "--bin", OBSERVER_BINARY])
    certification = build_executable(CERTIFICATION_PACKAGE,
                                     CERTIFICATION_PACKAGE.replace("-", "_"),
                                     ["test", "--lib", "--no-run"])
    listing = subprocess.run([str(certification), "--list"], cwd=ROOT,
                             capture_output=True, text=True, check=True)
    required = "c9_integrity_localization::c9_root_protocol_process_courtroom: test"
    if required not in listing.stdout.splitlines():
        raise SystemExit("Cargo-built certification binary omits the required courtroom")
    return run_courtroom(observer, certification)


if __name__ == "__main__":
    raise SystemExit(main())
