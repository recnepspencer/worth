#!/usr/bin/env python3
"""Fast local gate for the Query line and the crates beneath it.

    python scripts/ci/query_gate.py                  every suite
    python scripts/ci/query_gate.py relational,bank  the named suites
    python scripts/ci/query_gate.py --list           suite names

Why it is fast. Running `cargo test -p <one package>` once per package makes
cargo resolve features again for every selection, so the first-party chain is
rebuilt for each command. This driver builds each workspace's tests once
(`cargo test --no-run` over one selection), then runs the produced test
binaries directly: every listed test by exact name, in chunks, several
processes at a time. A red test in one binary does not hide the others, and
rerunning a few suites after a fix costs minutes.

Measured on an 8-core machine: the Bank workspace went from 717 s + 990 s as
one command per package to 230 s (140 s build, 90 s run) with the same 48
binaries and 477 tests; the Query packages other than Query Execution went
from 306 s to 83 s of run time (38 binaries, 4,441 tests).

sccache. When `sccache` is on PATH and no wrapper is configured, it is used.
It shares third-party crates across workspaces and checkouts (a cold root
test build went from 272 s to 187 s in a second checkout). Our own crates are
compiled incrementally and are not cached. Two things defeat it: sccache
refuses to run with CARGO_INCREMENTAL=1 in the environment (the driver drops
that variable; the profile default keeps our crates incremental), and it
hashes every CARGO_* variable into its key, so a CARGO_TARGET_DIR that
differs per checkout prevents sharing between checkouts.

Results go to target/query-gate (or --out): <suite>.json and one log per
build, chunk and command. The exit code is 0 only when every suite is green.
"""
import argparse
import json
import os
import shutil
import subprocess
import sys
import time
import tomllib
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

from query_gate_artifacts import test_binaries
from query_gate_diagnostics import failed_tests, report_failed_command
from query_gate_environment import environment

ROOT = Path(__file__).resolve().parents[2]
QUERY = "workspaces/worth-query/Cargo.toml"
FIXTURE = (
    "workspaces/worth-query/crates/worth-query-certification/fixtures/"
    "consumer_entry/Cargo.toml"
)
BANK = "workspaces/worth-query-bank-world/Cargo.toml"
UI = "workspaces/worth-ui/Cargo.toml"
BOUNDARY = "tools/boundary-check/Cargo.toml"

def packages(*names):
    return [part for name in names for part in ("-p", name)]


LOWER = ("worth-relational", "worth-execution", "worth-signal",
         "worth-runtime-bridge", "worth-runtime-world")
QUERY_PUBLIC = ("worth-query-execution", "worth-query-publication", "worth-query",
                "worth-query-host")
FIXTURE_PACKAGES = ("worth-query-topology-entry", "worth-query-parameter-entry",
                    "worth-query-consumer-root")

# One test build per group. Every suite of a group takes its binaries from
# the same cargo selection, so the first-party chain is compiled once.
ROOT_TESTS = [
    *packages("worth-relational", "worth-signal", "worth-execution",
              "worth-signal-wasm", "worth-server"),
    "--lib", "--tests",
]
GROUPS = {
    "root": ROOT_TESTS,
    "query": ["--manifest-path", QUERY, "--workspace", "--lib", "--tests"],
    "bank": ["--manifest-path", BANK, "--workspace", "--lib", "--bins", "--tests"],
    # Rust 1.94 repeatedly ICEs in this external schema host's incremental
    # evaluate_obligation cache. Keep the workaround in its build owner;
    # ordinary Query and Bank builds retain their incremental settings.
    "fixture": ["--manifest-path", FIXTURE, *packages(*FIXTURE_PACKAGES),
                "--lib", "--bins", "--config", "build.incremental=false"],
    "ui": ["--manifest-path", UI, "-p", "worth-ui-query-binding", "--lib"],
}

def lib_of(*names):
    return lambda binary: binary["package"] in names and binary["lib"]


def package_of(*names):
    return lambda binary: binary["package"] in names

def everything(_binary):
    return True


# suite -> (group, which binaries, tests per process, processes at once)
TEST_SUITES = {
    "relational": ("root", package_of("worth-relational"), 150, 4),
    "signal": ("root", package_of("worth-signal", "worth-signal-wasm"), 150, 4),
    "execution": ("root", package_of("worth-execution"), 150, 2),
    "query-execution": ("query", lib_of("worth-query-execution"), 80, 4),
    "query-rest": (
        "query", lambda binary: not lib_of("worth-query-execution")(binary), 150, 1,
    ),
    # Topology cases get individual processes; other fixture owners use normal chunks.
    "fixture": ("fixture", everything, 150, 16),
    "ui": ("ui", everything, 100, 2),
    "bank": ("bank", everything, 150, 1),
}
# Bank tests spawn the rail binary; a fresh target folder does not have it.
BEFORE = {
    "bank": [("bank-build-rail",
              ["cargo", "build", "--manifest-path", BANK, "-p", "bank-external-rail"])],
}

# Tests that read a process-wide allocation counter. Their own comments ask
# for a serial lane: beside any other test thread they count its allocations.
PROCESS_WIDE_PROBES = (
    "managed_run::tests::safe_point_observation::",
    "managed_run::tests::step_cost_bound::",
    "managed_run::tests::yield_cost_bound::",
    "read_set::source_facts::merging::",
    "tests::application_attempt::predecode_admission::",
    # The terminal-refusal fixture must use zero days, a process-wide counter
    # key also used by other document-retention scenarios.
    "producer_predicate_checkpoint::ordinary_source::terminal_unavailable::",
)

FIXTURE_CPU_EXCLUSIVE = (
    "checkpoint_recovery::parallel_history::matrix::table::required_wave_full_declared_matrix"
)

def clippy(*selection):
    return ["cargo", "clippy", *selection, "--all-targets", "--", "-D", "warnings"]

def bash():
    # On Windows a bare `bash` can resolve to the WSL launcher.
    git_bash = Path("C:/Program Files/Git/bin/bash.exe")
    return str(git_bash) if git_bash.exists() else (shutil.which("bash") or "bash")

def consumer_doctest_commands():
    configuration = tomllib.loads(
        (ROOT / "tools/boundary-check/config/consumer-docs.toml").read_text(encoding="utf-8")
    )
    return [
        ("doc-" + Path(selection["manifest"]).parent.name,
         ["cargo", "test", "--doc", "--no-fail-fast", "--jobs", "8",
          "--manifest-path", selection["manifest"], *packages(*selection["packages"])])
        for selection in configuration["doctests"]
    ] + [
        ("doc-owning-fixtures", ["cargo", "test", "--manifest-path", FIXTURE,
         "-p", "worth-query-consumer-root", "-p", "worth-query-topology-entry",
         "--bins", "--lib", "--jobs", "8", "--no-fail-fast", "--",
         "compiled_examples_match_consumer_documents"]),
        ("doc-owning-examples", ["cargo", "test", "--manifest-path", QUERY,
         *packages("worth-query-declaration", "worth-query-execution", "worth-query-publication",
                   "worth-query-host", "worth-query-certification"),
         "--example", "authored_workflow", "--example", "ordinary_product_workflow",
         "--jobs", "8", "--no-fail-fast"]),
    ]


# suite -> [(label, command)], one command at a time
COMMAND_SUITES = {
    "fmt": [
        ("fmt-root", ["cargo", "fmt", "--all", "--", "--check"]),
        ("fmt-query", ["cargo", "fmt", "--manifest-path", QUERY, "--all", "--", "--check"]),
        ("fmt-fixture",
         ["cargo", "fmt", "--manifest-path", FIXTURE, "--all", "--", "--check"]),
        ("fmt-bank", ["cargo", "fmt", "--manifest-path", BANK, "--all", "--", "--check"]),
    ],
    "lint-lower": [("clippy-lower", clippy(*packages(*LOWER)))],
    "lint-query": [
        ("clippy-query",
         clippy("--manifest-path", QUERY,
                *packages(*QUERY_PUBLIC, "worth-query-certification"))),
    ],
    "lint-fixture": [
        ("clippy-fixture", clippy("--manifest-path", FIXTURE, *packages(*FIXTURE_PACKAGES))),
    ],
    "lint-bank": [
        ("clippy-bank", clippy("--manifest-path", BANK, "--workspace")),
        ("clippy-ui-binding",
         clippy("--manifest-path", UI, *packages("worth-ui-query-binding"))),
    ],
    # Compiled doc examples: no other suite builds them.
    "doc-lower": [
        ("doc-root", ["cargo", "test", "--doc", "--no-fail-fast", *packages(*LOWER)]),
    ],
    "doc": consumer_doctest_commands(),
    "wasm": [
        ("wasm-check", ["cargo", "check", "-p", "worth-signal-wasm", "--all-targets"]),
    ],
    "boundary": [
        ("boundary-check",
         ["cargo", "run", "--manifest-path", BOUNDARY, "-p", "boundary-check",
          "--bin", "boundary-check"]),
        ("boundary-tests", ["cargo", "test", "--manifest-path", BOUNDARY]),
        ("agent-context",
         ["cargo", "run", "--manifest-path", "tools/agent-context/Cargo.toml",
          "-p", "agent-context", "--bin", "agent-context", "--", "check"]),
        ("line-caps", [bash(), "scripts/ci/check_workspace_rust_line_caps.sh", "dirty"]),
    ],
    "extra": [
        ("server-refused-read",
         ["cargo", "test", *ROOT_TESTS,
          "refused_basis_advancement_fails_the_slot_and_closes_the_lane"]),
        ("consumer-root",
         ["cargo", "run", "--manifest-path", FIXTURE, "-p", "worth-query-consumer-root"]),
        ("check-query-workspace",
         ["cargo", "check", "--manifest-path", QUERY, "--workspace", "--all-targets"]),
        ("check-ui-workspace",
         ["cargo", "check", "--manifest-path", UI, "--workspace", "--all-targets"]),
        ("check-root-workspace", ["cargo", "check", "--workspace", "--all-targets"]),
    ],
}

ORDER = ["fmt", "lint-lower", "lint-query", "lint-fixture", "lint-bank",
         "relational", "signal", "execution", "query-execution", "query-rest",
         "fixture", "ui", "bank", "doc-lower", "doc", "wasm", "boundary", "extra"]


class Gate:
    def __init__(self, out):
        self.out = out
        self.env = environment(ROOT)
        self.built = {}

    def command(self, label, argv):
        log = self.out / f"{label}.log"
        started = time.time()
        with open(log, "w", encoding="utf-8", errors="replace") as sink:
            code = subprocess.run(
                argv, cwd=ROOT, env=self.env, stdout=sink, stderr=subprocess.STDOUT
            ).returncode
        text = log.read_text(encoding="utf-8", errors="replace").splitlines()
        if code != 0:
            report_failed_command(label, code, log, text)
        return {
            "name": label, "exit": code, "seconds": round(time.time() - started),
            "results": [line for line in text if line.startswith("test result")],
            "failed": failed_tests(text), "log": log.name,
        }

    def build(self, group):
        """Build one group's tests once; return its test binaries."""
        if group in self.built:
            return self.built[group]
        log = self.out / f"group-{group}-build.log"
        started = time.time()
        with open(log, "w", encoding="utf-8") as sink:
            done = subprocess.run(
                ["cargo", "test", *GROUPS[group], "--no-run", "--message-format=json"],
                cwd=ROOT, env=self.env, stdout=subprocess.PIPE, stderr=sink, text=True,
                encoding="utf-8", errors="replace",
            )
        # Preserve JSON compiler diagnostics even when no test binary was produced.
        log.with_suffix(".jsonl").write_text(done.stdout, encoding="utf-8")
        binaries = test_binaries(done.stdout)
        ok = done.returncode == 0 and binaries
        self.built[group] = (binaries if ok else None, round(time.time() - started), log.name)
        return self.built[group]

    def chunk(self, suite, index, binary, names, extra=()):
        log = self.out / f"{suite}-chunk-{index:03}.log"
        env = dict(self.env, CARGO_MANIFEST_DIR=binary["cwd"])
        started = time.time()
        with open(log, "w", encoding="utf-8", errors="replace") as sink:
            code = subprocess.run(
                [binary["exe"], *(("--exact", *names, *extra) if binary["harness"] else ())],
                cwd=binary["cwd"], env=env,
                stdout=sink, stderr=subprocess.STDOUT,
            ).returncode
        text = log.read_text(encoding="utf-8", errors="replace").splitlines()
        return {
            "chunk": index, "binary": binary["package"] + ":" + binary["target"],
            "tests": len(names), "exit": code, "failed": failed_tests(text),
            "passed": sum(1 for line in text
                          if line.startswith("test ") and line.rstrip().endswith("... ok")),
            "seconds": round(time.time() - started), "log": log.name,
            "names": names if code != 0 else [],
        }

    def tests(self, suite):
        group, want, size, workers = TEST_SUITES[suite]
        for label, argv in BEFORE.get(suite, []):
            row = self.command(label, argv)
            if row["exit"] != 0:
                return {"suite": suite, "build": "FAILED", "log": row["log"], "ok": False}
        binaries, build_seconds, build_log = self.build(group)
        if binaries is None:
            return {"suite": suite, "build": "FAILED", "log": build_log, "ok": False}
        shared, alone, whole, exclusive, listing, total = [], [], [], [], [], 0
        for binary in filter(want, binaries):
            if not binary["harness"]:
                listing.append((binary["package"], binary["target"], 0))
                whole.append(binary)
                continue
            env = dict(self.env, CARGO_MANIFEST_DIR=binary["cwd"])
            listed = subprocess.run(
                [binary["exe"], "--list", "--format", "terse"], cwd=binary["cwd"],
                env=env, capture_output=True, text=True, encoding="utf-8",
                errors="replace",
            ).stdout
            names = sorted(line[: -len(": test")] for line in listed.splitlines()
                           if line.endswith(": test"))
            listing.append((binary["package"], binary["target"], len(names)))
            total += len(names)
            if suite == "fixture" and binary["package"] == "worth-query-topology-entry":
                # Every case can touch process-global providers, counters or ledgers.
                shared += [(binary, [name], ()) for name in names
                           if name != FIXTURE_CPU_EXCLUSIVE]
                if FIXTURE_CPU_EXCLUSIVE in names:
                    exclusive.append((binary, [FIXTURE_CPU_EXCLUSIVE], ()))
                continue
            probes = [n for n in names if any(mark in n for mark in PROCESS_WIDE_PROBES)]
            rest = [n for n in names if n not in set(probes)]
            shared += [(binary, rest[at:at + size], ()) for at in range(0, len(rest), size)]
            if probes:
                alone.append((binary, probes, ("--test-threads=1",)))
            # (the probes run after every shared chunk has ended)
        jobs = list(enumerate(
            shared + alone + [(binary, [], ()) for binary in whole] + exclusive
        ))
        started = time.time()

        def go(item):
            index, (binary, names, extra) = item
            return self.chunk(suite, index, binary, names, extra)

        workers = max(1, min(workers, (os.cpu_count() or 4) // 2))
        with ThreadPoolExecutor(max_workers=workers) as pool:
            results = list(pool.map(go, jobs[:len(shared)]))
        results += [go(item) for item in jobs[len(shared):]]
        failed = sorted({test for row in results for test in row["failed"]})
        bad_exit = [row["log"] for row in results if row["exit"] != 0 and not row["failed"]]
        return {
            "suite": suite, "tests": total, "binaries": listing,
            "passed": sum(row["passed"] for row in results), "failed": failed,
            "bad_exit": bad_exit, "build_seconds": build_seconds,
            "run_seconds": round(time.time() - started),
            "slowest": sorted(((row["seconds"], row["log"]) for row in results),
                              reverse=True)[:3],
            "ok": not failed and not bad_exit and bool(total or whole),
            "chunks": results,
        }

    def commands(self, suite):
        rows = [self.command(label, argv) for label, argv in COMMAND_SUITES[suite]]
        return {
            "suite": suite, "ok": all(row["exit"] == 0 for row in rows),
            "failed": [row["name"] for row in rows if row["exit"] != 0],
            "seconds": sum(row["seconds"] for row in rows), "commands": rows,
        }

    def run(self, suite):
        result = self.tests(suite) if suite in TEST_SUITES else self.commands(suite)
        (self.out / f"{suite}.json").write_text(json.dumps(result, indent=1),
                                                encoding="utf-8")
        return result

def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("suites", nargs="?", default=",".join(ORDER),
                        help="comma-separated suite names (default: all)")
    parser.add_argument("--out", type=Path, default=ROOT / "target" / "query-gate")
    parser.add_argument("--list", action="store_true", help="print suite names")
    args = parser.parse_args()
    if args.list:
        print("\n".join(ORDER))
        return 0
    suites = args.suites.split(",")
    unknown = [name for name in suites if name not in ORDER]
    if unknown:
        parser.error("unknown suite: " + ", ".join(unknown))
    args.out.mkdir(parents=True, exist_ok=True)
    gate = Gate(args.out)
    red = []
    for suite in suites:
        result = gate.run(suite)
        brief = {key: value for key, value in result.items()
                 if key not in ("chunks", "commands", "binaries")}
        print("GATE", json.dumps(brief), flush=True)
        if not result["ok"]:
            red.append(suite)
            if suite == "fmt":
                break
    print("GATE", "green" if not red else "red: " + ", ".join(red), flush=True)
    return 1 if red else 0


if __name__ == "__main__":
    sys.exit(main())
