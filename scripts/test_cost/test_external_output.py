"""External lane evidence must finish telemetry retention and build enforcement."""
import json
from pathlib import Path
import subprocess
from tempfile import TemporaryDirectory
from contextlib import ExitStack
import unittest
from unittest.mock import patch

from run_lane import execute, LaneExecutionFailure


class ExternalOutputTests(unittest.TestCase):
    def test_external_output_retains_telemetry_and_reaches_the_build_ratchet(self):
        with TemporaryDirectory() as temporary:
            root = Path(temporary).resolve() / 'checkout'
            root.mkdir()
            (root / 'Cargo.toml').write_text('[workspace]\nmembers=[]\n')
            for location in ('inside', 'outside'):
                with self.subTest(location=location), ExitStack() as stack:
                    output = root / 'target/out' if location == 'inside' else root.parent / 'evidence'
                    (output / 'build').mkdir(parents=True)
                    (output / 'build/native-selection.json').write_text('{"scope":"workspace"}')
                    (output / 'build/standalone.json').write_text('[]')
                    empty = {'rust-suites': {}}
                    result = {'lanes': {}, 'coverage': {'status': 'pass'}, 'source_sites': []}
                    violation = {'kind': 'deliberate-build-refusal'}

                    class Runner:
                        environment = {}
                        logs = [output / 'native.log']

                        def __call__(self, command, cwd, log):
                            self.logs.append(log)
                            return subprocess.CompletedProcess(command, 0, '', '')

                    replacements = {
                        'child_environment': {}, 'build': [],
                        'coverage': (empty, {'scale': empty}, {'status': 'pass'}),
                        'required_prebuilds': (None, False), 'prepare': {},
                        'original_reservations': {}, 'load_slots': {}, 'configuration': None,
                        'complete_coverage': ([], {}, {'status': 'pass'}),
                        'execution_observations': ([], {}),
                        'prove_executions': {'status': 'pass'},
                        'read_lane': {'cpu': [], 'headers': {'HEAD': 'a' * 40}},
                        'collect': result, 'compiler_observations': [],
                        'build_violations': [violation], 'markdown': 'complete',
                    }
                    mocks = {name: stack.enter_context(patch('run_lane.' + name, return_value=value))
                             for name, value in replacements.items()}
                    retained = stack.enter_context(patch('run_lane.retain_slots'))
                    with self.assertRaises(LaneExecutionFailure) as refused:
                        execute(root, 'root', ['owner-unit'], output, cargo_runner=Runner(), queries=[])
                    self.assertEqual(refused.exception.result['build_ratchet']['violations'], [violation])
                    mocks['build_violations'].assert_called_once()
                    self.assertEqual(Path(retained.call_args.args[3]), output / 'telemetry.log')
                    self.assertEqual(json.loads((output / 'build-ratchet.json').read_text()), [violation])
                    self.assertTrue((output / 'census.json').is_file())


if __name__ == '__main__':
    unittest.main()
