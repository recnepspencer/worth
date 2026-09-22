"""The native-platform proof lane.

The platform-pulse courtroom (`apps/platform-pulse/tests/executable_world`) is
the certified native world. On a host whose windowing system has a qualified
observer it EXECUTES against a live desktop; elsewhere it only COMPILES. This
module owns that host decision, the environment each execution needs, and the
report line a lane reader checks, so a dropped cfg flag can never certify at
exit 0: a certified build lists the serialized native worlds, and this lane
refuses to pass when it lists fewer than the declared floor.

Hosts:
  linux  execute under a private Xvfb server at the certified dpi with the
         `worth-ui-linux-x11-vulkan-software-v1` profile (x11 + software cfg
         arms, derived from `.cargo/config.toml` so `-C overflow-checks=on`
         survives; a bare RUSTFLAGS would have replaced it).
  win32  execute on the interactive desktop with the build's default profile.
  other  compile-only `cargo check` of the same test target.
"""

from __future__ import annotations

import enum
import glob
import os
import re
import shutil
import subprocess
import sys
import tomllib
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Callable, Mapping, Sequence

ROOT = Path(__file__).resolve().parents[2]
MANIFEST = ROOT / "workspaces/worth-ui/Cargo.toml"
CARGO_CONFIG = ROOT / ".cargo/config.toml"
LANE = "native-platform"
REPORT_TAG = "[worth-ui-native-platform]"
PACKAGE = "worth-ui-platform-pulse"
FEATURE = "executable-world"
TEST_TARGET = "executable_world"

# The certified X11 dpi: the courtroom's scale owner
# (`native_platform/linux_x11/scale.rs`) derives it as 96 * CERTIFIED_SCALE_MILLI
# / 1000 and asserts the display reports it, so the number has two readers that
# the self-test binds together. Xvfb receives it as `-dpi` (randr millimetres)
# and the resource database as `Xft.dpi`, which winit consults first.
CERTIFIED_X11_DPI = 144
# The server's screen: a 4K panel at 150 %, the class of host the Windows
# record was taken on. Without a window manager the product maps at the
# origin, so the screen must hold its canonical 1536x1024 logical window at the
# certified scale (2304x1536); the observer denies a smaller screen at
# qualification (`linux_x11/environment.rs`) and the self-test binds this
# constant to the product's extent constants.
CERTIFIED_X11_SCREEN = (3840, 2160)
XVFB_SCREEN = (
    f"-screen 0 {CERTIFIED_X11_SCREEN[0]}x{CERTIFIED_X11_SCREEN[1]}x24 "
    f"-dpi {CERTIFIED_X11_DPI} -noreset"
)
# One display is one desktop: the courtroom's worlds are serialized by the
# doctrine (`milestone-3.10.3.md`, certification command) and by the desktop
# lease, so the harness runs them one at a time.
HARNESS_ARGUMENTS = ("--test-threads=1", "--include-ignored")
CFG_AXES = ("worth_ui_windowing", "worth_ui_adapter")
X11_SOFTWARE_CFGS = ('worth_ui_windowing="x11"', 'worth_ui_adapter="software"')
LINUX_PROFILE = "worth-ui-linux-x11-vulkan-software-v1"
WINDOWS_PROFILE = "worth-ui-windows-dx12-v2"
# Changing --cfg invalidates every artifact in a target directory, so the X11
# software build keeps its own unless the operator names one; sharing it with
# any other lane would rebuild wgpu, naga and winit on every alternation.
LINUX_TARGET_DIR = ROOT / "workspaces/worth-ui/target/native-platform-x11-software"
WORKSPACE_TARGET_DIR = ROOT / "workspaces/worth-ui/target"
# Temporary instrumentation must carry this token so its residue stays findable in a
# built artifact after it has been removed from source. See `instrumentation_residue`.
INSTRUMENTATION_MARKER = b"WORTH-UI-TEMPORARY-INSTRUMENTATION"
# Variables the Linux execution must not inherit: RUSTFLAGS would compete with
# the derived encoded flags, and a scale override would counterfeit the dpi.
LINUX_REMOVED_VARIABLES = ("RUSTFLAGS", "WINIT_X11_SCALE_FACTOR")
X11_TOOLS = ("xvfb-run", "Xvfb", "xrdb")
# winit dlopens the first at event-loop construction and panics when it is
# missing; the first needs the second (DT_NEEDED). Packages libxkbcommon-x11-0
# and libxcb-xkb1.
X11_KEYBOARD_LIBRARIES = ("libxkbcommon-x11.so.0", "libxcb-xkb.so.1")
VULKAN_ICD_DIRECTORIES = (
    "/usr/share/vulkan/icd.d",
    "/etc/vulkan/icd.d",
    "/usr/local/share/vulkan/icd.d",
)
SOFTWARE_RASTERIZER_ICD = "lvp_icd*.json"
# The serialized native worlds the courtroom marks #[ignore] for the desktop:
# phases 2, 3, 6, 7, 8, gate D, F and F-reconstruction. A certified build lists
# at least these; a build that lost its certified cfg lists none.
CERTIFIED_WORLD_FLOOR = 8
LISTED_TEST = re.compile(r"^\S+: test$")
TEST_RESULT = re.compile(
    r"^test result: (?P<status>\w+)\. (?P<passed>\d+) passed; (?P<failed>\d+) failed; "
    r"(?P<ignored>\d+) ignored; (?P<measured>\d+) measured; (?P<filtered>\d+) filtered out"
)
PREFLIGHT_EXIT = 2
VERDICT_EXIT = 1


class Mode(enum.Enum):
    EXECUTE = "execute"
    COMPILE_ONLY = "compile-only"


@dataclass(frozen=True)
class Plan:
    mode: Mode
    profile: str | None
    environment: dict[str, str] = field(default_factory=dict)
    removed_variables: tuple[str, ...] = ()
    list_command: list[str] | None = None
    command: list[str] = field(default_factory=list)

    @property
    def commands(self) -> list[list[str]]:
        return [command for command in (self.list_command, self.command) if command]

    def child_environment(self, base: Mapping[str, str]) -> dict[str, str]:
        child = {key: value for key, value in base.items() if key not in self.removed_variables}
        child.update(self.environment)
        return child


def cargo_test_arguments(*harness_arguments: str) -> list[str]:
    return [
        "cargo", "test", "--manifest-path", str(MANIFEST),
        "-p", PACKAGE, "--features", FEATURE, "--test", TEST_TARGET,
        "--", *harness_arguments,
    ]


def build_rustflags(config: Path = CARGO_CONFIG) -> list[str]:
    with config.open("rb") as handle:
        return list(tomllib.load(handle).get("build", {}).get("rustflags", []))


def drop_cfg_axes(flags: list[str], axes: tuple[str, ...] = CFG_AXES) -> list[str]:
    """Remove every `--cfg <axis>=…` (two-token or `--cfg=` form) for the axes."""

    def names_axis(value: str) -> bool:
        return any(value.startswith(f"{axis}=") for axis in axes)

    kept: list[str] = []
    index = 0
    while index < len(flags):
        flag = flags[index]
        if flag == "--cfg" and index + 1 < len(flags) and names_axis(flags[index + 1]):
            index += 2
            continue
        if flag.startswith("--cfg=") and names_axis(flag[len("--cfg=") :]):
            index += 1
            continue
        kept.append(flag)
        index += 1
    return kept


def x11_software_rustflags(config: Path = CARGO_CONFIG) -> list[str]:
    flags = drop_cfg_axes(build_rustflags(config))
    for cfg in X11_SOFTWARE_CFGS:
        flags.extend(("--cfg", cfg))
    return flags


def plan(host: str = sys.platform, environ: Mapping[str, str] = os.environ) -> Plan:
    if host == "linux":
        environment = {"CARGO_ENCODED_RUSTFLAGS": "\x1f".join(x11_software_rustflags())}
        if "CARGO_TARGET_DIR" not in environ:
            environment["CARGO_TARGET_DIR"] = str(LINUX_TARGET_DIR)
        return Plan(
            Mode.EXECUTE,
            LINUX_PROFILE,
            environment,
            LINUX_REMOVED_VARIABLES,
            cargo_test_arguments("--list", "--ignored"),
            [
                "xvfb-run", "-a", "-s", XVFB_SCREEN,
                sys.executable, str(Path(__file__).resolve()),
                "--x11-session", str(CERTIFIED_X11_DPI), "--",
                *cargo_test_arguments(*HARNESS_ARGUMENTS),
            ],
        )
    if host == "win32":
        return Plan(
            Mode.EXECUTE,
            WINDOWS_PROFILE,
            list_command=cargo_test_arguments("--list", "--ignored"),
            command=cargo_test_arguments(*HARNESS_ARGUMENTS),
        )
    return Plan(
        Mode.COMPILE_ONLY,
        None,
        command=[
            "cargo", "check", "--manifest-path", str(MANIFEST),
            "-p", PACKAGE, "--features", FEATURE, "--test", TEST_TARGET,
        ],
    )


def shared_library_is_locatable(name: str, environ: Mapping[str, str]) -> bool:
    for directory in filter(None, environ.get("LD_LIBRARY_PATH", "").split(os.pathsep)):
        if (Path(directory) / name).exists():
            return True
    ldconfig = shutil.which("ldconfig") or "/sbin/ldconfig"
    try:
        listing = subprocess.run(
            [ldconfig, "-p"], capture_output=True, text=True, check=False
        ).stdout
    except OSError:
        return False
    return any(line.strip().split(" ", 1)[0] == name for line in listing.splitlines())


def software_rasterizer_icd_present(environ: Mapping[str, str]) -> bool:
    declared = environ.get("VK_DRIVER_FILES") or environ.get("VK_ICD_FILENAMES")
    if declared:
        return any(Path(entry).exists() for entry in declared.split(os.pathsep))
    return any(
        glob.glob(str(Path(directory) / SOFTWARE_RASTERIZER_ICD))
        for directory in VULKAN_ICD_DIRECTORIES
    )


def linux_preflight(environ: Mapping[str, str]) -> list[str]:
    failures = [f"missing tool on PATH: {tool}" for tool in X11_TOOLS if not shutil.which(tool)]
    failures.extend(
        f"missing shared library: {name} (install libxkbcommon-x11-0 and libxcb-xkb1 "
        "or put them on LD_LIBRARY_PATH)"
        for name in X11_KEYBOARD_LIBRARIES
        if not shared_library_is_locatable(name, environ)
    )
    if not software_rasterizer_icd_present(environ):
        failures.append(
            f"no software Vulkan ICD ({SOFTWARE_RASTERIZER_ICD}) under "
            f"{', '.join(VULKAN_ICD_DIRECTORIES)}; install mesa-vulkan-drivers (lavapipe)"
        )
    return failures


def run_capturing(command: list[str], environment: dict[str, str]) -> tuple[int, list[str]]:
    """Run a command, echoing stdout line by line while retaining it for parsing."""
    lines: list[str] = []
    try:
        with subprocess.Popen(
            command, cwd=ROOT, env=environment, stdout=subprocess.PIPE, text=True
        ) as process:
            assert process.stdout is not None
            for line in process.stdout:
                sys.stdout.write(line)
                sys.stdout.flush()
                lines.append(line.rstrip("\n"))
        return process.returncode, lines
    except OSError as error:
        print(f"{REPORT_TAG} cannot start {command[0]}: {error}", flush=True)
        return 127, lines


def parse_test_result(lines: list[str]) -> dict[str, int] | None:
    for line in reversed(lines):
        match = TEST_RESULT.match(line)
        if match:
            return {key: int(match.group(key)) for key in ("passed", "failed", "ignored")}
    return None


def target_directory(environment: Mapping[str, str]) -> Path:
    return Path(environment.get("CARGO_TARGET_DIR", WORKSPACE_TARGET_DIR))


def carries_marker(artifact: Path, marker: bytes = INSTRUMENTATION_MARKER) -> bool:
    """Scan a built artifact for the marker without reading it whole into memory."""
    overlap = len(marker) - 1
    tail = b""
    with artifact.open("rb") as handle:
        while chunk := handle.read(1 << 20):
            if marker in tail + chunk:
                return True
            tail = chunk[-overlap:]
    return False


def instrumentation_residue(directory: Path) -> list[str]:
    """Built world artifacts still carrying temporary instrumentation.

    Cargo decides freshness from mtime, so a source file restored to its original
    bytes can also be restored to its original mtime, leaving an instrumented
    artifact permanently fresh. The tree then reads clean while every later run
    executes code that is not in it. Source cannot show that; the artifact can.
    """
    deps = directory / "debug" / "deps"
    return [
        artifact.name
        for artifact in sorted(deps.glob(f"{TEST_TARGET}-*"))
        if not artifact.suffix and artifact.is_file() and carries_marker(artifact)
    ]


def verdict(
    worlds_listed: int,
    result: dict[str, int] | None,
    cargo_exit: int,
    residue: Sequence[str] = (),
) -> str:
    if residue:
        return "instrumentation-residue"
    if worlds_listed < CERTIFIED_WORLD_FLOOR:
        return "below-world-floor"
    if result is None:
        return "no-test-result"
    if result["ignored"] != 0:
        return "worlds-not-executed"
    if cargo_exit != 0 or result["failed"] != 0:
        return "failed"
    return "ok"


def render_report(evidence: dict[str, Any]) -> str:
    return REPORT_TAG + "".join(f" {key}={value}" for key, value in evidence.items())


def execute(
    active_plan: Plan, environ: Mapping[str, str] = os.environ
) -> tuple[int, dict[str, Any]]:
    evidence: dict[str, Any] = {"mode": active_plan.mode.value, "profile": active_plan.profile}
    environment = active_plan.child_environment(environ)
    if active_plan.mode is Mode.COMPILE_ONLY:
        exit_code, _ = run_capturing(active_plan.command, environment)
        evidence["result"] = "ok" if exit_code == 0 else "failed"
        print(render_report(evidence), flush=True)
        return exit_code, evidence
    failures = linux_preflight(environ) if sys.platform == "linux" else []
    if failures:
        for failure in failures:
            print(f"{REPORT_TAG} preflight: {failure}", flush=True)
        evidence["result"] = "preflight-failed"
        print(render_report(evidence), flush=True)
        return PREFLIGHT_EXIT, evidence
    assert active_plan.list_command is not None
    list_exit, listing = run_capturing(active_plan.list_command, environment)
    worlds_listed = sum(1 for line in listing if LISTED_TEST.match(line))
    evidence.update(worlds=worlds_listed, floor=CERTIFIED_WORLD_FLOOR)
    if list_exit != 0:
        evidence["result"] = "listing-failed"
        print(render_report(evidence), flush=True)
        return list_exit, evidence
    cargo_exit, output = run_capturing(active_plan.command, environment)
    result = parse_test_result(output)
    if result is not None:
        evidence.update(executed=result["passed"] + result["failed"], **result)
    residue = instrumentation_residue(target_directory(environment))
    for artifact in residue:
        print(f"{REPORT_TAG} instrumentation residue: {artifact}", flush=True)
    evidence["result"] = verdict(worlds_listed, result, cargo_exit, residue)
    print(render_report(evidence), flush=True)
    if evidence["result"] == "ok":
        return 0, evidence
    return (cargo_exit or VERDICT_EXIT), evidence


def describe(active_plan: Plan) -> None:
    for variable in active_plan.removed_variables:
        print(f"{REPORT_TAG} unset {variable}", flush=True)
    for key, value in active_plan.environment.items():
        print(f"{REPORT_TAG} {key}={value!r}", flush=True)
    for command in active_plan.commands:
        print(f"{REPORT_TAG}", subprocess.list2cmdline(command), flush=True)


def x11_session(dpi: str, command: list[str]) -> int:
    """Inside xvfb-run: publish the certified dpi to the server, then become the command."""
    xrdb = subprocess.run(
        ["xrdb", "-load", "-nocpp", "-"], input=f"Xft.dpi: {dpi}\n", text=True, check=False
    )
    if xrdb.returncode != 0:
        print(f"{REPORT_TAG} xrdb -load failed with {xrdb.returncode}", flush=True)
        return xrdb.returncode
    os.execvp(command[0], command)


def main(argv: list[str]) -> int:
    if argv[:1] == ["--x11-session"]:
        return x11_session(argv[1], argv[argv.index("--") + 1 :])
    active_plan = plan()
    if "--print-only" in argv:
        describe(active_plan)
        return 0
    exit_code, _ = execute(active_plan)
    return exit_code


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
