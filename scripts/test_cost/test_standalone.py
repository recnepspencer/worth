import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

from standalone import (StandaloneFailure, StandaloneFailureKind, discover, execute,
                        separate, synthetic_inventory)


class StandaloneProgramTests(unittest.TestCase):
    def package(self, root):
        manifest = root / 'Cargo.toml'
        manifest.write_text('[package]\nname="owner"\nversion="0.1.0"\n'
                            '[[example]]\nname="journey"\ntest=true\nharness=false\n'
                            '[[example]]\nname="not-a-test"\ntest=false\nharness=false\n'
                            '[[test]]\nname="ordinary"\n', encoding='utf-8')
        return manifest

    def artifact(self, manifest, name='journey', kind='example', **changes):
        event = dict(reason='compiler-artifact', manifest_path=str(manifest),
                     target={'name': name, 'kind': [kind]}, profile={'test': True},
                     fresh=True, executable=str(manifest.parent / name))
        event.update(changes)
        return event

    def test_manifest_and_real_artifact_select_only_harness_free_test_programs(self):
        with tempfile.TemporaryDirectory() as temporary:
            manifest = self.package(Path(temporary).resolve())
            rows = [self.artifact(manifest), self.artifact(manifest),
                    self.artifact(manifest, 'not-a-test', target={'name':'not-a-test','kind':['example'],'test':False}),
                    self.artifact(manifest, 'ordinary', 'test')]
            programs = discover('\n'.join(json.dumps(row) for row in rows))
            self.assertEqual(len(programs), 1)
            self.assertEqual(programs[0]['package'], 'owner')
            self.assertFalse(programs[0]['harness'])
            self.assertEqual(programs[0]['test'], '<main>')

    def test_nextest_metadata_binds_identity_and_is_filtered_before_libtest_listing(self):
        with tempfile.TemporaryDirectory() as temporary:
            manifest = self.package(Path(temporary).resolve())
            programs = discover([self.artifact(manifest)])
            original = {'rust-binaries': {'actual-main-id': {'binary-path': programs[0]['exe']},
                                          'ordinary-id': {'binary-path': 'ordinary'}}}
            filtered, selected = separate(original, programs)
            self.assertEqual(list(filtered['rust-binaries']), ['ordinary-id'])
            self.assertIn('actual-main-id', original['rust-binaries'])
            self.assertEqual(selected[0]['binary'], 'actual-main-id')
            inventory = synthetic_inventory(selected)
            self.assertFalse(inventory['rust-suites']['actual-main-id']['testcases']['<main>']['ignored'])
            self.assertFalse(inventory['rust-suites']['actual-main-id']['harness'])

    def test_ambiguous_unbound_graphs_are_not_arbitrarily_chosen(self):
        with tempfile.TemporaryDirectory() as temporary:
            manifest = self.package(Path(temporary).resolve())
            programs = discover([self.artifact(manifest),
                                 self.artifact(manifest, executable=str(manifest.parent / 'another'))])
            with self.assertRaises(StandaloneFailure) as failure:
                separate({'rust-binaries': {}}, programs)
            self.assertEqual(failure.exception.kind, StandaloneFailureKind.AMBIGUOUS_EXECUTABLE)

    def test_execution_uses_no_libtest_arguments_and_retains_nonzero_exit(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            program = dict(binary='owner::example/journey', package='owner', target='journey',
                           kind='example', exe=sys.executable, cwd=str(root))
            log = root / 'main.log'

            class Runner:
                environment = {'original': 'value'}
                def __call__(self, command, cwd, log_path):
                    self.command, self.cwd, self.child_environment = command, cwd, self.environment
                    Path(log_path).write_text('real command evidence\n', encoding='utf-8')
                    return subprocess.CompletedProcess(command, 7, '', '')

            runner = Runner()
            result = execute(program, runner, log, {'existing': 'value'})
            self.assertEqual(result.returncode, 7)
            self.assertEqual(runner.command[-1:], [sys.executable])
            self.assertEqual(len(runner.command), 3)
            self.assertEqual(runner.child_environment['NEXTEST_TEST_NAME'], '<main>')
            self.assertEqual(runner.child_environment['WORTH_TEST_COST_SCHEDULER'], 'standalone-process')
            self.assertNotIn('WORTH_TEST_COST_RESERVATIONS', runner.child_environment)
            self.assertFalse(log.with_suffix('.reservations.json').exists())
            self.assertEqual(runner.environment, {'original': 'value'})
            event = json.loads(log.read_text().splitlines()[-1])
            self.assertEqual(event['event'], 'failed')
            self.assertEqual(event['name'], 'owner::example/journey$<main>')

    def test_missing_executable_refuses_before_any_process_launch(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            with self.assertRaises(StandaloneFailure) as failure:
                execute(dict(exe=str(root/'missing')), None, root/'main.log', {})
            self.assertEqual(failure.exception.kind, StandaloneFailureKind.MISSING_EXECUTABLE)


if __name__ == '__main__':
    unittest.main()
