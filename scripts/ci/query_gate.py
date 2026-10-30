#!/usr/bin/env python3
"""Fast local gate for the Query line and the crates beneath it.

    python scripts/ci/query_gate.py                  every non-UI suite
    python scripts/ci/query_gate.py relational,bank  the named suites
    python scripts/ci/query_gate.py query-landing    all normal Query suites and docs
    python scripts/ci/query_gate.py bank-landing     all normal Bank suites and docs
    python scripts/ci/query_gate.py --list           suite names

Governed workspaces use one census/prebuild execution for every selected suite.
Category prefixes govern selection. External fixture and scheduled suites retain
their explicit process scheduling.

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
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

from query_gate_artifacts import test_binaries
from query_gate_diagnostics import failed_tests, report_failed_command
from query_gate_environment import environment
from query_gate_prebuild import GatePrebuild, PrerequisiteFailure, refused_scale_suites
from query_gate_cost import CostGate, DOC_SUITES, SUITE_ALIASES, expand_suites, governed
from cargo_capture import CargoRunner

from query_gate_courts import (
    BANK, BEFORE, BOUNDARY, FIXTURE, FIXTURE_PACKAGES, GROUPS, LOWER, QUERY,
    QUERY_PUBLIC, ROOT, ROOT_TESTS, TEST_SUITES, UI, packages, selected, CATEGORY_SUITES,
)


def in_class(name, cost_class):
    return cost_class in name.split("::")[:-1]


# The root Execution owner still needs one authority configuration per process.
# Its migration is owned by Track A1. Governed fixture tests use serial paths.
PROCESS_PER_CASE = {
    ("execution", "worth-execution"),
}

def clippy(*selection):
    return ["cargo", "clippy", *selection, "--all-targets", "--", "-D", "warnings"]

def bash():
    # On Windows a bare `bash` can resolve to the WSL launcher.
    for candidate in ("C:/Program Files/Git/bin/bash.exe", "C:/Program Files/Git/usr/bin/bash.exe"):
        if Path(candidate).exists():
            return candidate
    return shutil.which("bash") or "bash"

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
         clippy("--manifest-path", QUERY, "--workspace")),
    ],
    "lint-fixture": [
        ("clippy-fixture", clippy("--manifest-path", FIXTURE, *packages(*FIXTURE_PACKAGES))),
    ],
    "lint-bank": [
        ("clippy-bank", clippy("--manifest-path", BANK, "--workspace")),
        ("clippy-ui-binding",
         clippy("--manifest-path", UI, *packages("worth-ui-query-binding"))),
    ],
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
        ("consumer-root",
         ["cargo", "run", "--manifest-path", FIXTURE, "-p", "worth-query-consumer-root"]),
        ("check-query-workspace",
         ["cargo", "check", "--manifest-path", QUERY, "--workspace", "--all-targets"]),
        ("check-ui-workspace",
         ["cargo", "check", "--manifest-path", UI, "--workspace", "--all-targets"]),
        ("check-root-workspace", ["cargo", "check", "--workspace", "--all-targets"]),
    ],
}

# Environment one command adds over the gate's own.
COMMAND_ENV = {
    # sccache 0.18.0 inserts `/WX-` between cc 1.9.0's `-Tp` and its file, so
    # cl reads it as the source and libmimalloc-sys (via apps/platform-pulse)
    # fails to build. The empty wrapper also overrides `rustc-wrapper = sccache` in
    # ~/.cargo/config.toml; every other command keeps sccache.
    "check-ui-workspace": {"RUSTC_WRAPPER": ""},
}

ORDER = ["fmt", "lint-lower", "lint-query", "lint-fixture", "lint-bank",
         "relational", "signal", "execution", "root-rest", "query-execution", "query-rest",
         "query-process", "query-exhaustive", "fixture", "bank", "bank-process",
         "root-doc", "query-doc", "bank-doc", "wasm", "boundary", "extra"]
ORDER += [name for name in TEST_SUITES if name.startswith(("query-", "bank-", "root-"))
          and name not in ORDER and not name.endswith("-scale")]


class Gate:
    def __init__(self, out, census=False):
        self.out = out
        self.env = environment(ROOT)
        # An explicitly supplied build identity governs this complete gate.
        # Otherwise environment() issues one identity for all its child builds.
        if build_id := os.environ.get("WORTH_IMPLEMENTATION_BUILD_ID"):
            self.env["WORTH_IMPLEMENTATION_BUILD_ID"] = build_id
        self.built = {}
        self.prepared = set()
        self.census = census
        self.prerequisites = GatePrebuild(ROOT, out / "prebuilt", self.env)
        self.cost = CostGate(ROOT, out / "test-cost", self.env)

    def plan(self, suites):
        self.cost.configure(suites)

    def command(self, label, argv):
        log = self.out / f"{label}.log"
        started = time.time()
        child_environment = dict(self.env, **COMMAND_ENV.get(label, {}))
        if argv[:1] == ["cargo"]:
            code = CargoRunner(ROOT, child_environment)(argv, ROOT, log).returncode
        else:
            with open(log, "w", encoding="utf-8", errors="replace") as sink:
                code = subprocess.run(
                    argv, cwd=ROOT, env=child_environment,
                    stdout=sink, stderr=subprocess.STDOUT,
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
        done = CargoRunner(ROOT, self.env)(
            ["cargo", "test", *GROUPS[group], "--no-run", "--timings"], ROOT, log,
        )
        # Preserve JSON compiler diagnostics even when no test binary was produced.
        log.with_suffix(".jsonl").write_text(done.stdout, encoding="utf-8")
        binaries = test_binaries(done.stdout)
        ok = done.returncode == 0 and binaries
        self.built[group] = (binaries if ok else None, round(time.time() - started), log.name)
        return self.built[group]

    def chunk(self, suite, index, binary, names, extra=()):
        if refused_scale_suites([suite], self.census):
            raise ValueError("ignored scale tests are scheduled-only, outside census execution")
        if suite.endswith("-scale"):
            extra = (*extra, "--include-ignored")
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
        if refused_scale_suites([suite], self.census):
            raise ValueError("ignored scale tests are scheduled-only, outside census execution")
        if governed(suite):
            return self.cost.run(suite)
        return self._nonlanding_tests(suite)

    def _nonlanding_tests(self, suite):
        group, want, size, workers = TEST_SUITES[suite]
        for label, argv in BEFORE.get(suite, []):
            if label in self.prepared:
                continue
            row = self.command(label, argv)
            if row["exit"] != 0:
                return {"suite": suite, "build": "FAILED", "log": row["log"], "ok": False}
            self.prepared.add(label)
        binaries, build_seconds, build_log = self.build(group)
        if binaries is None:
            return {"suite": suite, "build": "FAILED", "log": build_log, "ok": False}
        shared, alone, whole, exclusive, listing, total = [], [], [], [], [], 0
        prerequisite_binaries = []
        for binary in filter(want, binaries):
            if not binary["harness"]:
                listing.append((binary["package"], binary["target"], 0))
                whole.append(binary)
                prerequisite_binaries.append(binary)
                continue
            env = dict(self.env, CARGO_MANIFEST_DIR=binary["cwd"])
            discovery = subprocess.run(
                [binary["exe"], "--list", "--format", "terse"], cwd=binary["cwd"],
                env=env, capture_output=True, text=True, encoding="utf-8",
                errors="replace",
            )
            if discovery.returncode:
                log = self.out / f"{suite}-{binary['package']}-{binary['target']}-list.log"
                log.write_text(discovery.stdout + discovery.stderr, encoding="utf-8")
                return {"suite": suite, "ok": False, "listing": "FAILED", "log": log.name}
            names = sorted(line[: -len(": test")] for line in discovery.stdout.splitlines()
                           if line.endswith(": test"))
            names = [name for name in names if selected(suite, binary, name)]
            if names:
                prerequisite_binaries.append(binary)
            listing.append((binary["package"], binary["target"], len(names)))
            total += len(names)
            exclusive_names = [n for n in names
                               if group == "fixture" and in_class(n, "exhaustive")]
            exclusive += [(binary, [name], ()) for name in exclusive_names]
            names = [n for n in names if n not in set(exclusive_names)]
            if (suite, binary["package"]) in PROCESS_PER_CASE:
                shared += [(binary, [name], ()) for name in names]
                continue
            probes = [n for n in names if in_class(n, "compile_fail")]
            isolated = [n for n in names if in_class(n, "serial") and n not in probes]
            rest = [n for n in names if n not in set(probes + isolated)]
            shared += [(binary, rest[at:at + size], ()) for at in range(0, len(rest), size)]
            shared += [(binary, [name], ()) for name in isolated]
            alone += [(binary, [name], ()) for name in probes]
            # Boundary outcomes are checked after independent chunks have finished.
        try:
            self.prerequisites.prepare(group, prerequisite_binaries)
        except PrerequisiteFailure as failure:
            return {"suite": suite, "ok": False, "prebuild": "FAILED",
                    "workspace": failure.workspace, "failure": str(failure)}
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
            "ok": not failed and not bad_exit and (bool(total or whole) or suite in CATEGORY_SUITES),
            "chunks": results,
        }

    def commands(self, suite):
        rows = [self.command(label, argv) for label, argv in COMMAND_SUITES[suite]
                if label not in ("check-ui-workspace", "clippy-ui-binding")]
        return {
            "suite": suite, "ok": all(row["exit"] == 0 for row in rows),
            "failed": [row["name"] for row in rows if row["exit"] != 0],
            "seconds": sum(row["seconds"] for row in rows), "commands": rows,
        }

    def run(self, suite):
        result = self.tests(suite) if suite in TEST_SUITES or suite in DOC_SUITES else self.commands(suite)
        (self.out / f"{suite}.json").write_text(json.dumps(result, indent=1),
                                                encoding="utf-8")
        return result

def main(argv=None, default_suites=None, default_out=None, allowed_suites=None):
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("suites", nargs="?", default=",".join(default_suites or ORDER),
                        help="suite names or query-landing, bank-landing, doc")
    parser.add_argument("--out", type=Path, default=default_out or ROOT / "target" / "query-gate")
    parser.add_argument("--list", action="store_true", help="print suite names")
    parser.add_argument("--census", action="store_true", help="refuse scheduled scale suites")
    args = parser.parse_args(argv)
    if args.list:
        print("\n".join(allowed_suites or dict.fromkeys([*ORDER, *TEST_SUITES, *SUITE_ALIASES, *COMMAND_SUITES])))
        return 0
    suites = expand_suites(args.suites.split(","))
    unknown = [name for name in suites if name not in TEST_SUITES and name not in COMMAND_SUITES and name not in DOC_SUITES]
    if unknown:
        parser.error("unknown suite: " + ", ".join(unknown))
    if len(set(suites)) != len(suites):
        parser.error("suite repeats in the execution plan")
    if allowed_suites is not None and any(name not in allowed_suites for name in suites):
        parser.error("suite is outside this workspace gate")
    if refused_scale_suites(suites, args.census):
        parser.error("ignored scale tests are scheduled-only, outside census execution")
    if args.census and "ui" in suites:
        parser.error("worth-ui is excluded from census execution")
    args.out.mkdir(parents=True, exist_ok=True)
    gate = Gate(args.out, census=args.census)
    gate.plan(suites)
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
