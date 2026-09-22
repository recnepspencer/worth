import os
import re
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest import TestCase, main
from unittest.mock import patch

import worth_ui_native_platform_lane as lane

COURTROOM = lane.ROOT / "workspaces/worth-ui/apps/platform-pulse/tests/executable_world"
NATIVE_PLATFORM = COURTROOM / "native_platform"
X11_SOFTWARE_FLAGS = ["--cfg", 'worth_ui_windowing="x11"', "--cfg", 'worth_ui_adapter="software"']


def rust_source(path: Path) -> str:
    return path.read_text(encoding="utf-8").replace("\r\n", "\n")


class NativePlatformLanePlanTests(TestCase):
    def test_linux_derives_encoded_rustflags_from_the_cargo_config(self) -> None:
        plan = lane.plan("linux", {})

        flags = plan.environment["CARGO_ENCODED_RUSTFLAGS"].split("\x1f")
        self.assertEqual(flags[:2], ["-C", "overflow-checks=on"])
        self.assertEqual(flags[-4:], X11_SOFTWARE_FLAGS)
        cfg_values = [flags[i + 1] for i, f in enumerate(flags) if f == "--cfg"]
        for axis in lane.CFG_AXES:
            self.assertEqual(sum(v.startswith(f"{axis}=") for v in cfg_values), 1, axis)
        self.assertIn("RUSTFLAGS", plan.removed_variables)
        self.assertIn("WINIT_X11_SCALE_FACTOR", plan.removed_variables)

    def test_linux_owns_a_target_directory_unless_the_operator_names_one(self) -> None:
        owned = lane.plan("linux", {})
        with TemporaryDirectory() as temporary:
            operator = lane.plan("linux", {"CARGO_TARGET_DIR": temporary})

        self.assertEqual(owned.environment["CARGO_TARGET_DIR"], str(lane.LINUX_TARGET_DIR))
        self.assertNotIn("CARGO_TARGET_DIR", operator.environment)

    def test_linux_executes_under_a_private_display_at_the_certified_dpi(self) -> None:
        plan = lane.plan("linux", {})

        self.assertIs(plan.mode, lane.Mode.EXECUTE)
        self.assertEqual(plan.profile, lane.LINUX_PROFILE)
        self.assertEqual(plan.command[:2], ["xvfb-run", "-a"])
        self.assertIn(f"-dpi {lane.CERTIFIED_X11_DPI} ", plan.command[3])
        self.assertIn("-noreset", plan.command[3])
        width, height = lane.CERTIFIED_X11_SCREEN
        self.assertIn(f"-screen 0 {width}x{height}x24 ", plan.command[3])
        session = plan.command.index("--x11-session")
        self.assertEqual(plan.command[session + 1], str(lane.CERTIFIED_X11_DPI))
        self.assertEqual(plan.command[-2:], ["--test-threads=1", "--include-ignored"])
        self.assertEqual(plan.list_command[-2:], ["--list", "--ignored"])
        for command in plan.commands:
            self.assertIn(lane.PACKAGE, command)
            self.assertIn(lane.FEATURE, command)
            self.assertIn(lane.TEST_TARGET, command)
            self.assertNotIn("--workspace", command)

    def test_child_environment_removes_competing_variables(self) -> None:
        plan = lane.plan("linux", {})

        child = plan.child_environment({"RUSTFLAGS": "-C x", "WINIT_X11_SCALE_FACTOR": "2", "K": "v"})

        self.assertNotIn("RUSTFLAGS", child)
        self.assertNotIn("WINIT_X11_SCALE_FACTOR", child)
        self.assertEqual(child["K"], "v")
        self.assertIn("CARGO_ENCODED_RUSTFLAGS", child)

    def test_windows_executes_on_the_interactive_desktop_without_a_wrapper(self) -> None:
        plan = lane.plan("win32", {})

        self.assertIs(plan.mode, lane.Mode.EXECUTE)
        self.assertEqual(plan.profile, lane.WINDOWS_PROFILE)
        self.assertEqual(plan.command[:2], ["cargo", "test"])
        self.assertEqual(plan.command[-2:], ["--test-threads=1", "--include-ignored"])
        self.assertEqual(plan.environment, {})

    def test_other_hosts_only_compile_the_courtroom(self) -> None:
        plan = lane.plan("darwin", {})

        self.assertIs(plan.mode, lane.Mode.COMPILE_ONLY)
        self.assertIsNone(plan.profile)
        self.assertIsNone(plan.list_command)
        self.assertEqual(plan.command[:2], ["cargo", "check"])
        self.assertIn(lane.TEST_TARGET, plan.command)

    def test_drop_cfg_axes_removes_both_token_forms_and_keeps_the_rest(self) -> None:
        flags = [
            "-C", "overflow-checks=on",
            "--cfg", 'worth_ui_windowing="wayland"',
            '--cfg=worth_ui_adapter="hardware"',
            "--cfg", "other_axis=\"x\"",
        ]

        self.assertEqual(
            lane.drop_cfg_axes(flags),
            ["-C", "overflow-checks=on", "--cfg", 'other_axis="x"'],
        )


class NativePlatformLaneVerdictTests(TestCase):
    def test_test_result_line_parses_the_last_harness_summary(self) -> None:
        result = lane.parse_test_result(
            ["noise", "test result: FAILED. 80 passed; 2 failed; 0 ignored; 0 measured; "
             "0 filtered out; finished in 9.10s"]
        )

        self.assertEqual(result, {"passed": 80, "failed": 2, "ignored": 0})
        self.assertIsNone(lane.parse_test_result(["running 82 tests"]))

    def test_verdict_refuses_a_build_that_lists_fewer_worlds_than_the_floor(self) -> None:
        clean = {"passed": 26, "failed": 0, "ignored": 0}

        self.assertEqual(lane.verdict(0, clean, 0), "below-world-floor")
        self.assertEqual(lane.verdict(lane.CERTIFIED_WORLD_FLOOR - 1, clean, 0), "below-world-floor")

    def test_verdict_refuses_worlds_left_ignored_or_missing_results(self) -> None:
        floor = lane.CERTIFIED_WORLD_FLOOR

        self.assertEqual(lane.verdict(floor, None, 0), "no-test-result")
        self.assertEqual(
            lane.verdict(floor, {"passed": 74, "failed": 0, "ignored": 8}, 0),
            "worlds-not-executed",
        )
        self.assertEqual(
            lane.verdict(floor, {"passed": 80, "failed": 2, "ignored": 0}, 101), "failed"
        )
        self.assertEqual(lane.verdict(floor, {"passed": 82, "failed": 0, "ignored": 0}, 0), "ok")

    def test_verdict_refuses_a_clean_run_whose_artifact_carries_instrumentation(self) -> None:
        """Residue outranks every count: they describe code that is not in the tree."""
        floor = lane.CERTIFIED_WORLD_FLOOR
        clean = {"passed": 82, "failed": 0, "ignored": 0}

        self.assertEqual(lane.verdict(floor, clean, 0), "ok")
        self.assertEqual(
            lane.verdict(floor, clean, 0, ["executable_world-45841bd787a4c978"]),
            "instrumentation-residue",
        )

    def test_residue_scan_names_only_the_world_artifacts_carrying_the_marker(self) -> None:
        with TemporaryDirectory() as directory:
            deps = Path(directory) / "debug" / "deps"
            deps.mkdir(parents=True)
            # Straddle a read boundary: a chunked scan must not lose the marker at the seam.
            seam = (1 << 20) - (len(lane.INSTRUMENTATION_MARKER) // 2)
            (deps / "executable_world-0000000000000001").write_bytes(
                b"\0" * seam + lane.INSTRUMENTATION_MARKER + b"\0" * 64
            )
            (deps / "executable_world-0000000000000002").write_bytes(b"\0" * (1 << 21))
            (deps / "executable_world-0000000000000001.d").write_bytes(
                lane.INSTRUMENTATION_MARKER
            )

            self.assertEqual(
                lane.instrumentation_residue(Path(directory)),
                ["executable_world-0000000000000001"],
            )

    def test_report_line_is_machine_readable(self) -> None:
        line = lane.render_report({"mode": "execute", "worlds": 8, "result": "ok"})

        self.assertEqual(line, f"{lane.REPORT_TAG} mode=execute worlds=8 result=ok")


class NativePlatformLaneRustBindingTests(TestCase):
    def test_certified_dpi_is_the_scale_the_courtroom_asserts(self) -> None:
        source = rust_source(NATIVE_PLATFORM / "mod.rs")
        match = re.search(r"const CERTIFIED_SCALE_MILLI: u32 = ([\d_]+);", source)
        self.assertIsNotNone(match)
        scale_milli = int(match.group(1).replace("_", ""))

        self.assertEqual(lane.CERTIFIED_X11_DPI, 96 * scale_milli // 1_000)

    def test_certified_screen_holds_the_product_window_at_the_certified_scale(self) -> None:
        source = rust_source(
            lane.ROOT / "workspaces/worth-ui/apps/platform-pulse/src/visual_identity_pulse.rs"
        )
        scale = rust_source(NATIVE_PLATFORM / "mod.rs")
        scale_milli = int(
            re.search(r"const CERTIFIED_SCALE_MILLI: u32 = ([\d_]+);", scale).group(1).replace("_", "")
        )
        extents = re.findall(
            r"const PLATFORM_PULSE_(?:CANONICAL|PRODUCT)_LOGICAL_EXTENT: \[u32; 2\] = \[(\d+), (\d+)\];",
            source,
        )
        self.assertEqual(len(extents), 2)

        for width, height in extents:
            self.assertLessEqual(int(width) * scale_milli // 1_000, lane.CERTIFIED_X11_SCREEN[0])
            self.assertLessEqual(int(height) * scale_milli // 1_000, lane.CERTIFIED_X11_SCREEN[1])

    def test_lane_profiles_are_the_ones_the_courtroom_certifies(self) -> None:
        source = rust_source(NATIVE_PLATFORM / "qualified_record.rs")

        self.assertIn(f'const CERTIFIED_PROFILE: &str = "{lane.LINUX_PROFILE}";', source)
        self.assertIn(f'const CERTIFIED_PROFILE: &str = "{lane.WINDOWS_PROFILE}";', source)

    def test_world_floor_counts_the_desktop_ignored_courtroom_worlds(self) -> None:
        ignored = sum(
            len(re.findall(r"^#\[ignore\b", rust_source(path), re.MULTILINE))
            for path in (COURTROOM / "courtroom").rglob("*.rs")
        )

        self.assertEqual(ignored, lane.CERTIFIED_WORLD_FLOOR)


if __name__ == "__main__":
    main()
