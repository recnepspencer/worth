import json
import os
from tempfile import TemporaryDirectory
from unittest import TestCase, main
from unittest.mock import patch

import run_worth_ui_test_lane as lane_runner


class WorthUiTestLaneTests(TestCase):
    def test_fast_lane_covers_all_feature_library_and_facade_proof(self) -> None:
        commands = lane_runner.commands_for("fast")

        self.assertEqual(len(commands), 2)
        for command in commands:
            self.assertIn("--all-features", command)
        self.assertIn("--lib", commands[0])
        self.assertIn("capability_contracts", commands[1])
        self.assertIn("registry_contracts", commands[1])

    def test_documentation_and_platform_lanes_have_exact_scope(self) -> None:
        documentation = lane_runner.commands_for("documentation")[0]
        platform = lane_runner.commands_for("platform-check")[0]

        self.assertIn("--doc", documentation)
        self.assertIn("--all-features", documentation)
        self.assertIn("--all-targets", platform)
        self.assertIn("--all-features", platform)

    def test_native_platform_lane_is_the_courtroom_owner_s_plan(self) -> None:
        commands = lane_runner.commands_for("native-platform")

        self.assertEqual(commands, lane_runner.native_platform_lane.plan().commands)
        for command in commands:
            self.assertIn("worth-ui-platform-pulse", command)
            self.assertIn("executable_world", command)
            self.assertNotIn("--workspace", command)

    def test_report_carries_lane_evidence_when_given(self) -> None:
        outcome = {"argv": ["x"], "command": "x", "duration_seconds": 0.5, "exit_code": 0, "error": None}
        with TemporaryDirectory() as temporary:
            with (
                patch.dict(os.environ, {"WORTH_UI_LANE_REPORT_DIR": temporary}),
                patch.object(lane_runner, "compiler_cache_stats", return_value=None),
            ):
                lane_runner.write_report(
                    "native-platform", [outcome], evidence={"mode": "execute", "worlds": 8}
                )
                lane_runner.write_report("fast", [outcome])
            native = json.loads(
                (lane_runner.Path(temporary) / "native-platform.json").read_text(encoding="utf-8")
            )
            fast = json.loads(
                (lane_runner.Path(temporary) / "fast.json").read_text(encoding="utf-8")
            )

        self.assertEqual(native["evidence"], {"mode": "execute", "worlds": 8})
        self.assertNotIn("evidence", fast)

    def test_filesystem_contract_reuses_the_application_contract_target(self) -> None:
        command = lane_runner.commands_for("filesystem-contract")[0]

        self.assertIn("--all-features", command)
        self.assertIn("worth-ui-certification", command)
        self.assertIn("application_contracts", command)
        self.assertIn("filesystem_", command)

    def test_closure_stress_is_visible_but_excluded_from_ordinary_execution(self) -> None:
        command = lane_runner.commands_for("closure-stress")[0]

        self.assertIn("application_contracts", command)
        self.assertIn("closure_stress_", command)
        self.assertIn("--ignored", command)
        self.assertNotIn("--test-threads", command)
        self.assertLess(command.index("--manifest-path"), command.index("--"))

    def test_compile_contracts_use_the_dedicated_two_session_runner(self) -> None:
        command = lane_runner.commands_for("compile-contracts")[0]

        self.assertEqual(command[0], lane_runner.sys.executable)
        self.assertTrue(command[1].endswith("run_worth_ui_compile_contracts.py"))

    def test_compile_contracts_also_run_the_in_crate_probes(self) -> None:
        command = lane_runner.commands_for("compile-contracts")[1]

        self.assertEqual(command[0], lane_runner.sys.executable)
        self.assertTrue(command[1].endswith("run_worth_ui_compile_probes.py"))

    def test_full_lane_retains_every_independent_proof_family(self) -> None:
        commands = lane_runner.commands_for("full")
        rendered = [" ".join(command) for command in commands]

        self.assertIn("--workspace", commands[0])
        self.assertIn("--all-features", commands[0])
        self.assertTrue(any("application_contracts" in command for command in rendered))
        self.assertTrue(any("compile_contracts" in command for command in rendered))
        self.assertTrue(any("--doc" in command for command in rendered))
        self.assertTrue(any("host_contract_only_adapter" in command for command in rendered))

    def test_dependency_contract_honors_an_isolated_cargo_target(self) -> None:
        with TemporaryDirectory() as temporary, patch.dict(
            os.environ, {"CARGO_TARGET_DIR": temporary}
        ):
            command = lane_runner.dependency_contract_commands()[0]

        target_argument = command[command.index("--target-dir") + 1]
        self.assertEqual(target_argument, temporary)

    def test_report_is_machine_readable_and_preserves_failed_command(self) -> None:
        outcomes = [
            {
                "argv": ["cargo", "test"],
                "command": "cargo test",
                "duration_seconds": 1.25,
                "exit_code": 1,
                "error": None,
            }
        ]
        with TemporaryDirectory() as temporary:
            with (
                patch.dict(os.environ, {"WORTH_UI_LANE_REPORT_DIR": temporary}),
                patch.object(lane_runner, "compiler_cache_stats", return_value=None),
            ):
                lane_runner.write_report("fast", outcomes)

            payload = json.loads(
                (lane_runner.Path(temporary) / "fast.json").read_text(encoding="utf-8")
            )

        self.assertFalse(payload["success"])
        self.assertEqual(payload["commands"][0]["exit_code"], 1)
        self.assertEqual(payload["total_duration_seconds"], 1.25)
        self.assertEqual(payload["execution_posture"], "sequential")

    def test_full_report_records_bounded_parallel_wall_time(self) -> None:
        outcomes = [
            {
                "argv": ["cargo", "test"],
                "command": "cargo test",
                "duration_seconds": 4.0,
                "exit_code": 0,
                "error": None,
            }
        ]
        with TemporaryDirectory() as temporary:
            with (
                patch.dict(os.environ, {"WORTH_UI_LANE_REPORT_DIR": temporary}),
                patch.object(lane_runner, "compiler_cache_stats", return_value=None),
            ):
                lane_runner.write_report("full", outcomes, total_duration_seconds=2.0)
            payload = json.loads(
                (lane_runner.Path(temporary) / "full.json").read_text(encoding="utf-8")
            )

        self.assertEqual(payload["total_duration_seconds"], 2.0)
        self.assertEqual(
            payload["execution_posture"],
            "bounded_parallel_independent_proof_families",
        )


if __name__ == "__main__":
    main()
