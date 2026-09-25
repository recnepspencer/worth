from __future__ import annotations

import argparse
import concurrent.futures
import json
import os
import subprocess
import sys
import time
from datetime import UTC, datetime
from pathlib import Path
from typing import Any

import worth_ui_native_platform_lane as native_platform_lane


ROOT = Path(__file__).resolve().parents[2]
MANIFEST = ROOT / "workspaces/worth-ui/Cargo.toml"
DEPENDENCY_FIXTURE = (
    ROOT
    / "workspaces/worth-ui/crates/worth-ui-certification/tests/fixtures"
    / "host_contract_only_adapter/Cargo.toml"
)

CERTIFICATION_TARGETS = (
    "application_contracts",
    "declaration_contracts",
    "graph_contracts",
    "inspection_contracts",
    "measurement_contracts",
    "obligation_contracts",
    "topology_contracts",
)
FULL_LANE_WORKERS = 2


def cargo(*arguments: str) -> list[str]:
    command = ["cargo", *arguments]
    manifest = ["--manifest-path", str(MANIFEST)]
    if "--" not in command:
        return [*command, *manifest]
    separator = command.index("--")
    return [*command[:separator], *manifest, *command[separator:]]


def fast_commands() -> list[list[str]]:
    return [
        cargo("test", "--workspace", "--all-features", "--lib"),
        cargo(
            "test",
            "--all-features",
            "-p",
            "worth-ui",
            "--test",
            "capability_contracts",
            "--test",
            "registry_contracts",
        ),
    ]


def documentation_commands() -> list[list[str]]:
    return [cargo("test", "--workspace", "--all-features", "--doc")]


def platform_check_commands() -> list[list[str]]:
    return [cargo("check", "--workspace", "--all-targets", "--all-features")]


def filesystem_contract_commands() -> list[list[str]]:
    return [
        cargo(
            "test",
            "--all-features",
            "-p",
            "worth-ui-certification",
            "--test",
            "application_contracts",
            "filesystem_",
        )
    ]


def closure_stress_commands() -> list[list[str]]:
    return [
        cargo(
            "test",
            "--all-features",
            "-p",
            "worth-ui-certification",
            "--test",
            "application_contracts",
            "closure_stress_",
            "--",
            "--ignored",
            "--nocapture",
        )
    ]


def compile_contract_commands() -> list[list[str]]:
    return [
        [sys.executable, str(ROOT / "scripts/ci/run_worth_ui_compile_contracts.py")],
        [sys.executable, str(ROOT / "scripts/ci/run_worth_ui_compile_probes.py")],
    ]


def certification_commands() -> list[list[str]]:
    command = cargo("test", "--all-features", "-p", "worth-ui-certification")
    for target in CERTIFICATION_TARGETS:
        command.extend(("--test", target))
    replacement_storm = cargo(
        "test",
        "--all-features",
        "-p",
        "worth-ui-runtime",
        "--lib",
        "long_replacement_storm_",
        "--",
        "--ignored",
    )
    return [command, replacement_storm]


def dependency_contract_commands() -> list[list[str]]:
    configured_target = os.environ.get("CARGO_TARGET_DIR")
    target = (
        Path(configured_target)
        if configured_target
        else ROOT / "workspaces/worth-ui/target/dependency-contracts"
    )
    return [
        [
            "cargo",
            "check",
            "--manifest-path",
            str(DEPENDENCY_FIXTURE),
            "--target-dir",
            str(target),
        ]
    ]


def full_commands() -> list[list[str]]:
    return [
        cargo(
            "test",
            "--workspace",
            "--all-features",
            "--lib",
            "--bins",
            "--examples",
        ),
        *fast_commands()[1:],
        *certification_commands(),
        *compile_contract_commands(),
        *documentation_commands(),
        *dependency_contract_commands(),
    ]


def commands_for(lane: str) -> list[list[str]]:
    if lane == "fast":
        return fast_commands()
    if lane == "documentation":
        return documentation_commands()
    if lane == "compile-contracts":
        return compile_contract_commands()
    if lane == "hostile-certification":
        return certification_commands()
    if lane == "dependency-contract":
        return dependency_contract_commands()
    if lane == "platform-check":
        return platform_check_commands()
    if lane == native_platform_lane.LANE:
        return native_platform_lane.plan().commands
    if lane == "filesystem-contract":
        return filesystem_contract_commands()
    if lane == "closure-stress":
        return closure_stress_commands()
    if lane == "full":
        return full_commands()
    raise ValueError(f"unknown lane: {lane}")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Run one explicit Worth UI proof lane")
    parser.add_argument(
        "lane",
        choices=(
            "fast",
            "documentation",
            "compile-contracts",
            "hostile-certification",
            "dependency-contract",
            "platform-check",
            native_platform_lane.LANE,
            "filesystem-contract",
            "closure-stress",
            "full",
        ),
    )
    parser.add_argument(
        "--print-only",
        action="store_true",
        help="print the exact commands without executing them",
    )
    return parser.parse_args()


def compiler_cache_stats() -> dict[str, Any] | None:
    wrapper = os.environ.get("RUSTC_WRAPPER", "")
    if Path(wrapper).name.lower() not in {"sccache", "sccache.exe"}:
        return None
    try:
        result = subprocess.run(
            ["sccache", "--show-stats"],
            cwd=ROOT,
            env=os.environ.copy(),
            capture_output=True,
            text=True,
            check=False,
        )
    except OSError as error:
        return {"available": False, "error": str(error)}
    return {
        "available": result.returncode == 0,
        "exit_code": result.returncode,
        "stdout": result.stdout,
        "stderr": result.stderr,
    }


def write_report(
    lane: str,
    outcomes: list[dict[str, Any]],
    total_duration_seconds: float | None = None,
    evidence: dict[str, Any] | None = None,
) -> None:
    configured_directory = os.environ.get("WORTH_UI_LANE_REPORT_DIR")
    if configured_directory is None:
        return
    report_directory = Path(configured_directory)
    if not report_directory.is_absolute():
        report_directory = ROOT / report_directory
    report_directory.mkdir(parents=True, exist_ok=True)
    payload = {
        "schema_version": 1,
        "lane": lane,
        "captured_at": datetime.now(UTC).isoformat(),
        "platform": sys.platform,
        "success": all(outcome["exit_code"] == 0 for outcome in outcomes),
        "total_duration_seconds": round(
            total_duration_seconds
            if total_duration_seconds is not None
            else sum(float(outcome["duration_seconds"]) for outcome in outcomes),
            3,
        ),
        "execution_posture": "bounded_parallel_independent_proof_families"
        if lane == "full"
        else "sequential",
        "commands": outcomes,
        "compiler_cache": compiler_cache_stats(),
    }
    if evidence is not None:
        payload["evidence"] = evidence
    destination = report_directory / f"{lane}.json"
    destination.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")


def main() -> int:
    args = parse_args()
    commands = commands_for(args.lane)
    if args.print_only:
        if args.lane == native_platform_lane.LANE:
            native_platform_lane.describe(native_platform_lane.plan())
            return 0
        for command in commands:
            print("[worth-ui-test-lane]", subprocess.list2cmdline(command), flush=True)
        return 0
    if args.lane == "full":
        return run_parallel_full_lane(commands)
    if args.lane == native_platform_lane.LANE:
        return run_native_platform_lane()

    outcomes: list[dict[str, Any]] = []
    for command in commands:
        print("[worth-ui-test-lane]", subprocess.list2cmdline(command), flush=True)
        outcome = run_command(command)
        outcomes.append(outcome)
        if outcome["exit_code"] != 0:
            write_report(args.lane, outcomes)
            return int(outcome["exit_code"])
    write_report(args.lane, outcomes)
    return 0


def run_native_platform_lane() -> int:
    """The native-platform lane owns its environment, display and verdict; this
    records its outcome in the same report shape as every other lane."""
    plan = native_platform_lane.plan()
    native_platform_lane.describe(plan)
    started = time.perf_counter()
    exit_code, evidence = native_platform_lane.execute(plan)
    outcome = {
        "argv": plan.command,
        "command": subprocess.list2cmdline(plan.command),
        "duration_seconds": round(time.perf_counter() - started, 3),
        "exit_code": exit_code,
        "error": None,
    }
    write_report(native_platform_lane.LANE, [outcome], evidence=evidence)
    return exit_code


def run_parallel_full_lane(commands: list[list[str]]) -> int:
    started = time.perf_counter()
    for command in commands:
        print("[worth-ui-test-lane]", subprocess.list2cmdline(command), flush=True)
    with concurrent.futures.ThreadPoolExecutor(max_workers=FULL_LANE_WORKERS) as executor:
        outcomes = list(executor.map(run_command, commands))
    duration = time.perf_counter() - started
    write_report("full", outcomes, duration)
    return next(
        (int(outcome["exit_code"]) for outcome in outcomes if outcome["exit_code"] != 0),
        0,
    )


def run_command(command: list[str]) -> dict[str, Any]:
    rendered = subprocess.list2cmdline(command)
    started = time.perf_counter()
    try:
        result = subprocess.run(command, cwd=ROOT, env=os.environ.copy(), check=False)
        exit_code = result.returncode
        error = None
    except OSError as execution_error:
        exit_code = 127
        error = str(execution_error)
    return {
        "argv": command,
        "command": rendered,
        "duration_seconds": round(time.perf_counter() - started, 3),
        "exit_code": exit_code,
        "error": error,
    }


if __name__ == "__main__":
    raise SystemExit(main())
