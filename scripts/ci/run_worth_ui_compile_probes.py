"""Run the Worth UI runtime's in-crate compile probes.

Sealed truth is crate-private, so the public compile-contract fixture cannot
reach it. The runtime instead carries probe cases gated by
`worth_ui_compile_probe` beside the owners they probe. Each case is announced
by a `// expect: compiles` or `// expect: E0000` line directly above its
`#[cfg(worth_ui_compile_probe = "name")]` attribute, and runs to the next case.

Every `compiles` case is built in one session, which must emit no error or
warning. Each refused case is built alone, because one refusal can stop the
compiler before another is reached, and must fail with its expected error
code, raised inside its own case and nowhere else.
"""

from __future__ import annotations

import json
import os
import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
WORKSPACE = ROOT / "workspaces/worth-ui"
RUNTIME = WORKSPACE / "crates/worth-ui-runtime/src"
PROBE_SOURCES = (
    RUNTIME / "compile_probes.rs",
    RUNTIME / "mounting/presentation/motion_sampling/sampling/compile_probe.rs",
    RUNTIME
    / "mounting/presentation/work_producer/scroll_motion_groups/compile_probe.rs",
)
CASE = re.compile(
    r'^// expect: (compiles|E\d{4})\r?\n#\[cfg\(worth_ui_compile_probe = "([a-z0-9-]+)"\)\]',
    re.MULTILINE,
)


@dataclass(frozen=True)
class Probe:
    name: str
    expected: str | None
    source: Path
    first_line: int
    last_line: int


def probes_in(source: Path, text: str) -> list[Probe]:
    """The cases in one probe source, each spanning to the next case."""
    starts = [
        (text.count("\n", 0, found.start()) + 1, found) for found in CASE.finditer(text)
    ]
    last = text.count("\n") + 1
    probes = []
    for index, (line, found) in enumerate(starts):
        end = starts[index + 1][0] - 1 if index + 1 < len(starts) else last
        expected = None if found.group(1) == "compiles" else found.group(1)
        probes.append(Probe(found.group(2), expected, source, line, end))
    return probes


def load_probes() -> list[Probe]:
    probes: list[Probe] = []
    for source in PROBE_SOURCES:
        found = probes_in(source, source.read_text(encoding="utf-8"))
        if not any(probe.expected is None for probe in found) or not any(
            probe.expected for probe in found
        ):
            raise SystemExit(f"{source}: needs a compiling case beside each refused one")
        probes.extend(found)
    names = [probe.name for probe in probes]
    duplicates = sorted({name for name in names if names.count(name) > 1})
    if duplicates:
        raise SystemExit(f"duplicate compile probe cases: {', '.join(duplicates)}")
    return probes


def compile_cases(names: list[str]) -> tuple[int, list[dict[str, object]]]:
    command = [
        "cargo",
        "rustc",
        "--locked",
        "-p",
        "worth-ui-runtime",
        "--lib",
        "--profile",
        "check",
        "--message-format=json",
        "--color",
        "never",
        "--",
        "--cfg",
        "worth_ui_compile_probe",
    ]
    for name in names:
        command.extend(("--cfg", f'worth_ui_compile_probe="{name}"'))
    environment = os.environ.copy()
    environment.setdefault("CARGO_TARGET_DIR", str(WORKSPACE / "target"))
    process = subprocess.run(
        command, cwd=WORKSPACE, text=True, capture_output=True, env=environment
    )
    messages = []
    for line in process.stdout.splitlines():
        try:
            message = json.loads(line)
        except json.JSONDecodeError:
            continue
        if message.get("reason") == "compiler-message":
            messages.append(message["message"])
    if process.returncode != 0 and not messages:
        messages.append({"level": "error", "message": process.stderr.strip()})
    return process.returncode, messages


def primary_line(message: dict[str, object]) -> tuple[Path, int] | None:
    for span in message.get("spans") or []:
        if span.get("is_primary"):
            return (WORKSPACE / span["file_name"]).resolve(), span["line_start"]
    return None


def rendered(message: dict[str, object]) -> str:
    return str(message.get("rendered") or message.get("message"))


def compiling_failures(status: int, messages: list[dict[str, object]]) -> list[str]:
    failures = [
        rendered(message)
        for message in messages
        if message.get("level") in ("error", "warning")
    ]
    if status != 0 and not failures:
        failures.append(f"compiling cases exited {status}")
    return failures


def refusal_failures(
    probe: Probe, status: int, messages: list[dict[str, object]]
) -> list[str]:
    errors = [
        message
        for message in messages
        if message.get("level") == "error"
        and not str(message.get("message")).startswith("aborting due to")
    ]
    if status == 0 or not errors:
        return [f"{probe.name}: compiled, but must fail with {probe.expected}"]
    failures = []
    for error in errors:
        code = (error.get("code") or {}).get("code")
        place = primary_line(error)
        inside = place is not None and (
            place[0] == probe.source.resolve()
            and probe.first_line <= place[1] <= probe.last_line
        )
        if code != probe.expected or not inside:
            failures.append(
                f"{probe.name}: expected only {probe.expected} inside the case\n"
                f"{rendered(error)}"
            )
    return failures


def main() -> int:
    probes = load_probes()
    compiling = [probe.name for probe in probes if probe.expected is None]
    failures = compiling_failures(*compile_cases(compiling))
    refused = [probe for probe in probes if probe.expected]
    for probe in refused:
        failures.extend(refusal_failures(probe, *compile_cases([probe.name])))
    if failures:
        print("\n\n".join(failures), file=sys.stderr)
        return 1
    print(
        f"Worth UI compile probes passed: {len(compiling)} compiling cases, "
        f"{len(refused)} refused cases"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
