import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

from census import main
from compiler_units import CompilerUnitRefusal, declarations
from ratchet import build_violations, compiler_observations
from test_compiler_units import event, graph, unit


class CensusCommandCaptureTests(unittest.TestCase):
    def test_direct_command_profile_override_is_refused_before_declaring_or_launching_cargo(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            manifest, output = self.fixture(root)
            command = ['cargo', 'check', '--config', 'profile.dev.debug=0']
            argv, repo, cwd = self.cli(root, manifest, output, command)
            with argv, repo, cwd, patch('census.CargoRunner') as runner, patch('census.run') as launch:
                self.assertEqual(main(), 1)
            runner.assert_not_called()
            launch.assert_not_called()
            result = json.loads((output/'census.json').read_text())
            self.assertEqual(result['lanes'], {})
            self.assertIn('omit --config overrides', result['capture_refusal'])

    def test_external_profile_override_refuses_cargo_and_wrappers_without_reusing_old_captures(self):
        for command in (['cargo', 'check'], [sys.executable, 'gate.py']):
            with self.subTest(command=command), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary).resolve()
                manifest, output = self.fixture(root)
                (output/'lane.log').write_text(self.body(['cargo', 'check', '--message-format=json']))
                argv, repo, cwd = self.cli(root, manifest, output, command)
                with argv, repo, cwd, patch.dict(os.environ, {'CARGO_PROFILE_TEST_DEBUG': '0'}), \
                     patch('census.CargoRunner') as runner, patch('census.run') as launch:
                    self.assertEqual(main(), 1)
                runner.assert_not_called()
                launch.assert_not_called()
                result = json.loads((output/'census.json').read_text())
                self.assertEqual(result['lanes'], {})
                self.assertIn('CARGO_PROFILE_TEST_DEBUG', result['capture_refusal'])

    def fixture(self, root):
        manifest = root/'Cargo.toml'
        manifest.write_text('[package]\nname="fixture-court-capture"\nversion="0.1.0"\n')
        output = root/'target/test-cost/capture'
        output.mkdir(parents=True)
        return manifest, output

    def body(self, command):
        owner = unit('owner')
        marker = {'type': 'cargo-invocation', 'command': command, 'mode': 'check', 'cargo_profile': 'dev',
                  'unit_plan': {'units': declarations(graph(owner)), 'compiler_inputs': 'flags'}}
        return 'Command: ' + json.dumps(command) + '\n\n' + '\n'.join(map(json.dumps, [marker,
            dict(event(owner), reason='compiler-artifact', fresh=False), {'reason': 'build-finished', 'success': True}])) + '\nExit code: 0\n'

    def cli(self, root, manifest, output, command):
        return patch('sys.argv', ['census.py', '--manifest', str(manifest), '--output', str(output), '--', *command]), \
               patch('census.repository_root', return_value=root), patch('census.Path.cwd', return_value=root)

    def test_direct_cargo_uses_planned_runner_and_deduplicates_only_its_known_alias(self):
        for repeated in (False, True):
            with self.subTest(repeated=repeated), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary).resolve()
                manifest, output = self.fixture(root)
                test, calls = self, []
                class Runner:
                    def __init__(self, repository, environment):
                        self.environment, self.logs = environment, []
                        test.assertEqual(repository, root)
                    def __call__(self, command, cwd, path):
                        calls.append((command, cwd))
                        declaration, actual = output/'declaration.log', output/'retained-actual.log'
                        declaration.write_text('Command: ' + json.dumps(command + ['--unit-graph']) + '\nExit code: 0\n')
                        body = test.body(command)
                        actual.write_text(body)
                        path.write_text(body)
                        self.logs = [declaration, actual]
                        indexed = [declaration, actual, path, actual]
                        if repeated:
                            other = output/'separate-real-invocation.log'
                            other.write_text(body)
                            indexed.append(other)
                        Path(self.environment['WORTH_TEST_COST_CAPTURE_INDEX']).write_text(
                            '\n'.join(json.dumps({'log': str(log)}) for log in indexed))
                        return subprocess.CompletedProcess(command, 0, '', '')
                argv, repo, cwd = self.cli(root, manifest, output, ['cargo', 'check'])
                with argv, repo, cwd, patch('census.CargoRunner', Runner), patch('census.run') as plain:
                    self.assertEqual(main(), 0)
                plain.assert_not_called()
                self.assertEqual(calls, [(['cargo', 'check', '--message-format=json'], root)])
                result = json.loads((output/'census.json').read_text())
                self.assertEqual(len(result['lanes']), 3 if repeated else 2)
                self.assertEqual(sum(len(lane['artifacts']) for lane in result['lanes'].values()), 2 if repeated else 1)
                self.assertEqual(bool(build_violations(compiler_observations(result['lanes'].values(), output),
                                                       {'max_compiles_per_unit': 1})), repeated)

    def test_direct_preflight_refusal_never_reuses_an_old_canonical_capture(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            manifest, output = self.fixture(root)
            (output/'lane.log').write_text(self.body(['cargo', 'check', '--message-format=json']))
            class Runner:
                def __init__(self, repository, environment):
                    self.logs, self.environment = [], environment
                def __call__(self, command, cwd, path):
                    declaration = output/'new-refused-declaration.log'
                    declaration.write_text('Command: ' + json.dumps(command + ['--unit-graph']) + '\nExit code: 101\n')
                    self.logs.append(declaration)
                    Path(self.environment['WORTH_TEST_COST_CAPTURE_INDEX']).write_text(json.dumps({'log': str(declaration)}))
                    raise CompilerUnitRefusal('new declaration refused before Cargo compilation')
            argv, repo, cwd = self.cli(root, manifest, output, ['cargo', 'check'])
            with argv, repo, cwd, patch('census.CargoRunner', Runner):
                self.assertEqual(main(), 1)
            result = json.loads((output/'census.json').read_text())
            self.assertEqual(len(result['lanes']), 1)
            self.assertNotIn('command', result['lanes'])
            self.assertEqual(sum(len(lane['artifacts']) for lane in result['lanes'].values()), 0)
            self.assertIn('new declaration refused', result['capture_refusal'])

    def test_non_cargo_wrapper_keeps_plain_capture_and_distinct_nested_invocations(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            manifest, output = self.fixture(root)
            def wrapper(command, path, environment):
                path.write_text('Command: ' + json.dumps(command) + '\nExit code: 0\n')
                paths = [output/'child-one.log', output/'child-two.log']
                for child in paths:
                    child.write_text(self.body(['cargo', 'check']))
                Path(environment['WORTH_TEST_COST_CAPTURE_INDEX']).write_text(
                    '\n'.join(json.dumps({'log': str(child)}) for child in paths))
                return 0
            argv, repo, cwd = self.cli(root, manifest, output, ['python', 'gate.py'])
            with argv, repo, cwd, patch('census.CargoRunner') as runner, patch('census.run', side_effect=wrapper):
                self.assertEqual(main(), 0)
            runner.assert_not_called()
            result = json.loads((output/'census.json').read_text())
            self.assertEqual(len(result['lanes']), 3)
            self.assertEqual(sum(len(lane['artifacts']) for lane in result['lanes'].values()), 2)


if __name__ == '__main__':
    unittest.main()
