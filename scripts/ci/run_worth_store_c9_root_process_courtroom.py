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


def build_observer() -> pathlib.Path:
    command = [
        "cargo",
        "build",
        "--manifest-path",
        str(MANIFEST),
        "-p",
        OBSERVER_PACKAGE,
        "--bin",
        OBSERVER_BINARY,
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
            and target.get("name") == OBSERVER_BINARY
            and "bin" in target.get("kind", [])
            and message.get("executable")
        ):
            executable = pathlib.Path(message["executable"])
    if result.returncode != 0:
        sys.stdout.write(result.stdout)
        sys.stderr.write(result.stderr)
        raise SystemExit(result.returncode)
    if executable is None or not executable.is_file():
        raise SystemExit("Cargo did not report the freshly built observer executable")
    return executable.resolve()


def run_courtroom(observer: pathlib.Path) -> int:
    environment = os.environ.copy()
    environment[OBSERVER_ENV] = str(observer)
    owner_tests = [
        "cargo",
        "test",
        "--manifest-path",
        str(MANIFEST),
        "-p",
        CERTIFICATION_PACKAGE,
        "c9_integrity_localization",
    ]
    owner_result = subprocess.run(owner_tests, cwd=ROOT, env=environment)
    if owner_result.returncode != 0:
        return owner_result.returncode
    family_graph = owner_tests[:-1] + [
        "c9_integrity_localization::offline_process::independent_observer_traverses_current_family_graph",
        "--",
        "--exact",
        "--ignored",
        "--nocapture",
    ]
    family_result = subprocess.run(family_graph, cwd=ROOT, env=environment)
    if family_result.returncode != 0:
        return family_result.returncode
    courtroom = [
        "cargo",
        "test",
        "--manifest-path",
        str(MANIFEST),
        "-p",
        CERTIFICATION_PACKAGE,
        "c9_integrity_localization::c9_root_protocol_process_courtroom",
        "--",
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
    exact_success = (
        "test c9_integrity_localization::c9_root_protocol_process_courtroom ... ok"
    )
    if exact_success not in courtroom_result.stdout:
        sys.stderr.write(
            "C.9 courtroom command exited successfully without executing the exact test\n"
        )
        return 1
    return 0


def main() -> int:
    return run_courtroom(build_observer())


if __name__ == "__main__":
    raise SystemExit(main())
