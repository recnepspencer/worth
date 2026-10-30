import unittest
import os
from pathlib import Path
from tempfile import TemporaryDirectory
from types import SimpleNamespace
from unittest.mock import patch

from lane_environment import LaneEnvironmentRefusal
from run_lane import LANDING, child_environment, execute, selected_lanes, log_label


class GovernedExecutionPlanTests(unittest.TestCase):
    def test_rust_plan_identities_have_windows_safe_log_paths(self):
        self.assertEqual(log_label('scenario::nextest'), 'scenario--nextest')
        self.assertLessEqual(len(log_label('smoke::' + 'domain' * 100)), 80)

    def test_scheduled_scale_is_never_an_executable_landing_lane(self):
        self.assertNotIn('scale', LANDING)
        with self.assertRaises(ValueError):
            selected_lanes('scale')

    def test_repeated_partition_is_refused_before_execution(self):
        with self.assertRaises(ValueError):
            selected_lanes('process,process')

    def test_environment_keeps_every_core_and_uses_committed_compiler_profiles(self):
        with patch.dict(os.environ, {'CARGO_BUILD_JOBS': '24'}, clear=True):
            environment = child_environment()
        self.assertEqual(environment['CARGO_BUILD_JOBS'], '24')
        self.assertFalse(any(key.startswith('CARGO_PROFILE_') for key in environment))
        self.assertNotIn('CARGO_INCREMENTAL', environment)
        self.assertNotIn('RUSTUP_TOOLCHAIN', environment)
        self.assertEqual(environment['WORTH_IMPLEMENTATION_BUILD_ID'],
                         '00000000-0000-4000-8000-000000000001')

    def test_supplied_gate_profile_overrides_are_refused_before_source_discovery_or_build(self):
        with TemporaryDirectory() as temporary:
            output = Path(temporary)/'unused'
            runner = SimpleNamespace(environment={'CARGO_PROFILE_TEST_DEBUG': '0'})
            with patch('run_lane.collect') as discovery, patch('run_lane.build') as build:
                with self.assertRaises(LaneEnvironmentRefusal):
                    execute(Path.cwd(), 'store', [], output, cargo_runner=runner)
            discovery.assert_not_called()
            build.assert_not_called()
            self.assertFalse(output.exists())


if __name__ == '__main__':
    unittest.main()
