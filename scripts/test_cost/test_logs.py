import json
from pathlib import Path
import tempfile
import unittest

from logs import duplicates, llvm_lines, read_lane
from telemetry import timing_summary


def compiled(**changes):
    value = dict(reason='compiler-artifact', package_id='path+file:///x#owner@1',
                 target={'name': 'owner', 'kind': ['lib']}, features=[],
                 profile={'opt_level': '0', 'test': False}, fresh=False,
                 filenames=['C:/x/target/debug/deps/owner-123.rlib'])
    value.update(changes)
    return value


class ArtifactObservationTests(unittest.TestCase):
    def test_plain_libtest_summaries_retain_counts_without_inventing_test_identities(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'legacy.log'
            path.write_text('test result: ok. 3 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out\n'
                            'test result: FAILED. 5 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out\n')
            lane = read_lane(path)
            self.assertEqual(lane['tests'], [])
            self.assertEqual(lane['plain_test_counts'], {'pass': 8, 'fail': 1, 'ignored': 2})
            self.assertEqual(timing_summary(lane)['pass'], 8)

    def test_aggregate_phase_wall_is_distinct_from_the_aggregation_command_duration(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'phases.log'
            path.write_text('Start: 2026-10-29T00:00:00+00:00\n' + json.dumps(dict(
                type='test-cost-phases', build_seconds=50, wall_seconds=240)) +
                '\nEnd: 2026-10-29T00:00:01+00:00\n')
            lane = read_lane(path)
            self.assertEqual(lane['telemetry']['wall_seconds'], 240)
            self.assertEqual(lane['telemetry']['run_seconds'], 190)
            self.assertEqual(lane['telemetry']['capture_wall_seconds'], 1)

    def lane(self, rows):
        with tempfile.TemporaryDirectory() as directory:
            log = Path(directory) / 'lane.log'
            log.write_text('\n'.join(json.dumps(r) for r in rows), encoding='utf-8')
            return read_lane(log)

    def test_fresh_artifacts_do_not_count_as_compilation(self):
        result = self.lane([compiled(), compiled(fresh=True), compiled(features=['cert'])])
        self.assertEqual(len(result['artifacts']), 2)
        self.assertEqual(result['duplicates'][0]['causes'], ['features'])
        warm = self.lane([compiled(fresh=True)])
        self.assertEqual(warm['build_observation'], 'json')
        self.assertEqual(warm['artifacts'], [])

    def test_profile_target_directory_and_check_drift_are_distinguished(self):
        result = self.lane([compiled(),
                            dict(type='cargo-invocation', target_dir='alternate', mode='check'),
                            compiled(profile={'test': False, 'opt_level': '2'}, filenames=['alternate/debug/deps/owner.rmeta'])])
        self.assertEqual(result['duplicates'][0]['causes'],
                         ['profile', 'target_dir', 'check_mode'])

    def test_one_cargo_check_can_link_a_host_dependency_and_check_its_target_library(self):
        result = self.lane([dict(type='cargo-invocation', mode='check'), compiled(),
                            compiled(filenames=['C:/x/target/debug/deps/owner-456.rmeta'])])
        self.assertEqual([row['check_mode'] for row in result['artifacts']], ['build', 'check'])
        self.assertEqual([row['invocation_mode'] for row in result['artifacts']], ['check', 'check'])
        self.assertEqual(result['duplicates'][0]['causes'], ['check_mode'])

    def test_equal_own_features_do_not_prove_identical_dependency_or_compiler_inputs(self):
        result = self.lane([compiled(), compiled(filenames=['C:/x/target/debug/deps/owner-456.rlib'])])
        self.assertEqual(result['duplicates'][0]['causes'], ['unattributed compiler inputs'])
        repeated = self.lane([compiled(), compiled()])
        self.assertEqual(repeated['duplicates'][0]['causes'], ['repeated observed compiler unit'])

    def test_text_only_build_is_unknown_rather_than_zero_duplicates(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'lane.log'
            path.write_text('   Compiling owner v1\n')
            self.assertEqual(read_lane(path)['build_observation'], 'unmeasured')

    def test_formal_build_and_run_telemetry_are_distinct_from_summed_test_work(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'formal.log'
            path.write_text('Finished `ci-test` profile [unoptimized] target(s) in 26m 39s\n'
                            'Summary [   0.265s] 2 tests run: 2 passed\n')
            observation = read_lane(path)
            self.assertEqual(observation['telemetry']['build_seconds'], 1599)
            self.assertEqual(observation['telemetry']['run_seconds'], .265)
            self.assertEqual(observation['build_observation'], 'unmeasured')

    def test_cargo_json_inside_test_output_is_also_observed(self):
        result = self.lane([dict(type='test', event='failed', name='compiler',
                                 stdout=json.dumps(compiled()), exec_time=900)])
        self.assertEqual(len(result['artifacts']), 1)
        self.assertEqual(result['telemetry']['test_work_seconds'], 900)

    def test_nextest_ignored_inventory_counts_do_not_require_an_executed_test_event(self):
        result = self.lane([dict(type='suite', event='ok', passed=1, failed=0, ignored=1)])
        self.assertEqual(timing_summary(result)['ignored'], 1)
        self.assertEqual(timing_summary(result)['pass'], 0)

    def test_llvm_totals_and_generic_helper_families_are_retained(self):
        rows = llvm_lines('1000 (100%) 30 (100%) TOTAL\n500 (50%) 4 (10%) kit::world<T>',
                          ['kit::world'])
        self.assertEqual(rows[0]['lines'], 1000)
        self.assertEqual(rows[1]['testkit_helpers'], ['kit::world'])
        with self.assertRaises(ValueError):
            llvm_lines('error: no such command')


if __name__ == '__main__':
    unittest.main()
