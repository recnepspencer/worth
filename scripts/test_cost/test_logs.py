import json
from pathlib import Path
import tempfile
import unittest

from logs import duplicates, llvm_lines, read_lane


def compiled(**changes):
    value = dict(reason='compiler-artifact', package_id='path+file:///x#owner@1',
                 target={'name': 'owner', 'kind': ['lib']}, features=[],
                 profile={'opt_level': '0', 'test': False}, fresh=False,
                 filenames=['C:/x/target/debug/deps/owner-123.rlib'])
    value.update(changes)
    return value


class ArtifactObservationTests(unittest.TestCase):
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
                            compiled(profile={'test': False, 'opt_level': '2'})])
        self.assertEqual(result['duplicates'][0]['causes'],
                         ['profile', 'target_dir', 'check_mode'])

    def test_text_only_build_is_unknown_rather_than_zero_duplicates(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'lane.log'
            path.write_text('   Compiling owner v1\n')
            self.assertEqual(read_lane(path)['build_observation'], 'unmeasured')

    def test_cargo_json_inside_test_output_is_also_observed(self):
        result = self.lane([dict(type='test', event='failed', name='compiler',
                                 stdout=json.dumps(compiled()), exec_time=900)])
        self.assertEqual(len(result['artifacts']), 1)
        self.assertEqual(result['telemetry']['test_work_seconds'], 900)

    def test_llvm_totals_and_generic_helper_families_are_retained(self):
        rows = llvm_lines('1000 (100%) 30 (100%) TOTAL\n500 (50%) 4 (10%) kit::world<T>',
                          ['kit::world'])
        self.assertEqual(rows[0]['lines'], 1000)
        self.assertEqual(rows[1]['testkit_helpers'], ['kit::world'])
        with self.assertRaises(ValueError):
            llvm_lines('error: no such command')


if __name__ == '__main__':
    unittest.main()
