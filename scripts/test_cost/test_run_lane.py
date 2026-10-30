import unittest

from run_lane import LANDING, child_environment, selected_lanes, log_label


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

    def test_environment_keeps_every_core_and_disables_debug_information(self):
        environment = child_environment()
        self.assertEqual(environment['CARGO_PROFILE_DEV_DEBUG'], '0')
        self.assertEqual(environment['CARGO_PROFILE_TEST_DEBUG'], '0')
        self.assertEqual(environment.get('CARGO_BUILD_JOBS'), __import__('os').environ.get('CARGO_BUILD_JOBS'))


if __name__ == '__main__':
    unittest.main()
