#!/usr/bin/env python3
"""Run TLC model checking or NDJSON trace validation with zero external dependencies.

Usage:
  python tools/tla/run_tlc.py check specs/tla/<subsystem>/<Model>.tla [workers]
  python tools/tla/run_tlc.py trace <TraceSpec.tla> <trace.ndjson>

Locates `java` (PATH or ~/.local/tools/tla/jre/bin/java[.exe]) and
`tla2tools.jar` (TLA2TOOLS_JAR env var, ~/.local/tools/tla/tla2tools.jar,
or /opt/homebrew/opt/tla-plus/libexec/tla2tools.jar).
"""

from __future__ import annotations

import json
import os
import pathlib
import shutil
import subprocess
import sys
import tempfile


def find_java() -> str:
    env_java = os.environ.get("TLA_JAVA")
    if env_java and pathlib.Path(env_java).is_file():
        return env_java
    home = pathlib.Path.home()
    preferred = [
        home / ".local" / "tools" / "tla" / "jre" / "bin" / "java.exe",
        home / ".local" / "tools" / "tla" / "jre" / "bin" / "java",
        pathlib.Path("/opt/homebrew/opt/openjdk@21/bin/java"),
        pathlib.Path("/opt/homebrew/opt/openjdk/bin/java"),
    ]
    for c in preferred:
        if c.is_file():
            return str(c)
    in_path = shutil.which("java")
    if in_path:
        return in_path
    raise SystemExit("ERROR: java not found. Set TLA_JAVA or install JRE 21.")


def find_tla2tools() -> str:
    env_jar = os.environ.get("TLA2TOOLS_JAR")
    if env_jar and pathlib.Path(env_jar).is_file():
        return env_jar
    home = pathlib.Path.home()
    candidates = [
        home / ".local" / "tools" / "tla" / "tla2tools.jar",
        pathlib.Path("/opt/homebrew/share/tla2tools.jar"),
        pathlib.Path("/usr/local/share/tla2tools.jar"),
    ]
    for c in candidates:
        if c.is_file():
            return str(c)
    raise SystemExit("ERROR: tla2tools.jar not found. Set TLA2TOOLS_JAR or download into ~/.local/tools/tla/tla2tools.jar.")


def to_tla_literal(value: object) -> str:
    if isinstance(value, bool):
        return "TRUE" if value else "FALSE"
    if isinstance(value, int):
        return str(value)
    if isinstance(value, str):
        escaped = value.replace("\\", "\\\\").replace('"', '\\"')
        return f'"{escaped}"'
    if isinstance(value, list):
        return "<<" + ", ".join(to_tla_literal(v) for v in value) + ">>"
    if isinstance(value, dict):
        if "__set__" in value and len(value) == 1:
            return "{" + ", ".join(to_tla_literal(v) for v in value["__set__"]) + "}"
        fields = [f"{k} |-> {to_tla_literal(v)}" for k, v in value.items()]
        return "[" + ", ".join(fields) + "]"
    raise TypeError(f"Unsupported JSON value for TLA+ literal: {value!r}")


def run_check(tla_path: pathlib.Path, cfg_path: pathlib.Path | None, workers: str) -> int:
    java = find_java()
    jar = find_tla2tools()
    cfg = cfg_path or tla_path.with_suffix(".cfg")
    if not cfg.is_file():
        raise SystemExit(f"ERROR: cfg not found: {cfg}")
    with tempfile.TemporaryDirectory(prefix="tlc-meta-") as metadir:
        cmd = [
            java,
            "-XX:+UseParallelGC",
            "-cp",
            jar,
            "tlc2.TLC",
            "-cleanup",
            "-metadir",
            metadir,
            "-workers",
            workers,
            "-config",
            str(cfg),
            str(tla_path),
        ]
        print("Running:", " ".join(cmd), flush=True)
        proc = subprocess.run(cmd, cwd=str(tla_path.parent))
        return proc.returncode


def run_trace(trace_tla: pathlib.Path, ndjson_path: pathlib.Path) -> int:
    rows = []
    with ndjson_path.open("r", encoding="utf-8") as fh:
        for line in fh:
            line = line.strip()
            if line:
                rows.append(json.loads(line))
    if not rows:
        raise SystemExit(f"ERROR: empty trace file: {ndjson_path}")

    with tempfile.TemporaryDirectory(prefix="tlc-trace-") as tmp:
        tmp_dir = pathlib.Path(tmp)
        for p in trace_tla.parent.glob("*.tla"):
            shutil.copy2(p, tmp_dir / p.name)
        for p in trace_tla.parent.glob("*.cfg"):
            shutil.copy2(p, tmp_dir / p.name)

        data_tla = tmp_dir / "TraceData.tla"
        data_tla.write_text(
            "---- MODULE TraceData ----\n"
            "EXTENDS Integers, Sequences, FiniteSets\n"
            f"Trace == {to_tla_literal(rows)}\n"
            "====\n",
            encoding="utf-8",
        )
        return run_check(tmp_dir / trace_tla.name, tmp_dir / trace_tla.with_suffix(".cfg").name, "1")


def main(argv: list[str]) -> int:
    if len(argv) < 3:
        print(__doc__)
        return 2
    mode = argv[1]
    if mode == "check":
        tla_path = pathlib.Path(argv[2]).resolve()
        workers = argv[3] if len(argv) > 3 else "auto"
        return run_check(tla_path, None, workers)
    if mode == "trace":
        if len(argv) < 4:
            print(__doc__)
            return 2
        return run_trace(pathlib.Path(argv[2]).resolve(), pathlib.Path(argv[3]).resolve())
    print(__doc__)
    return 2


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
