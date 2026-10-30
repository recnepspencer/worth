import json
from pathlib import Path
from subprocess import CompletedProcess
import tempfile
import unittest

from proof_prebuild import (DENIED_PACKAGES, FIXTURES, LEGAL_PACKAGES, ProofPrebuildError,
                            checked_packages, package_name, prepare)


def package_messages(denied=DENIED_PACKAGES):
    rows = []
    for name in LEGAL_PACKAGES + DENIED_PACKAGES:
        row = {'package_id': f'path+file:///fixture/{name}#0.1.0'}
        if name in denied:
            row.update(reason='compiler-message', message={'level': 'error', 'message': 'band rejected',
                                                          'rendered': f'guard diagnostic for {name}'})
        else:
            row.update(reason='compiler-artifact', fresh=False, target={'name': name, 'kind': ['lib']})
        rows.append(json.dumps(row))
    return '\n'.join(rows)


class ProofPrebuildTests(unittest.TestCase):
    def test_batched_check_retains_distinct_package_observations(self):
        result = checked_packages(CompletedProcess([], 101, package_messages(), 'aggregate progress'))
        self.assertEqual([name for name in LEGAL_PACKAGES if result[name]['outcome'] == 'compiled'],
                         list(LEGAL_PACKAGES))
        for name in DENIED_PACKAGES:
            self.assertEqual(result[name]['outcome'], 'denied')
            self.assertIn(name, result[name]['stderr'])
            for other in set(DENIED_PACKAGES) - {name}:
                self.assertNotIn(other, result[name]['stderr'])

    def test_unobserved_package_cannot_be_mistaken_for_a_successful_check(self):
        rows = package_messages().splitlines()[1:]
        with self.assertRaisesRegex(ProofPrebuildError, 'no package observation'):
            checked_packages(CompletedProcess([], 101, '\n'.join(rows), 'failed dependency'))

    def test_unexpected_success_is_retained_for_the_court_to_reject(self):
        result = checked_packages(CompletedProcess([], 0, package_messages(denied=()), ''))
        self.assertEqual(result['empty-prefix-consumer']['outcome'], 'compiled')

    def test_package_identity_accepts_cargo_ids_with_renamed_paths(self):
        self.assertEqual(package_name('path+file:///a/different#worth-entry-consumer@0.1.0'),
                         'worth-entry-consumer')
        self.assertEqual(package_name('path+file:///a/worth-entry-consumer#0.1.0'),
                         'worth-entry-consumer')

    def fixture(self, temporary):
        repository = Path(temporary).resolve()
        path = repository / FIXTURES / 'Cargo.toml'
        path.parent.mkdir(parents=True)
        path.write_text('[workspace]\nmembers=[]')
        (repository / 'Cargo.lock').write_text('version = 4\n')
        return repository

    def test_preparation_uses_one_target_and_independent_raw_ir_files(self):
        with tempfile.TemporaryDirectory() as temporary:
            repository = self.fixture(temporary)
            output = repository / 'target/test-cost/proof'
            commands = []

            def runner(command, cwd, log):
                commands.append(command)
                self.assertEqual(cwd, repository)
                if command[1] == 'check':
                    return CompletedProcess(command, 101, package_messages(), 'expected aggregate refusal')
                if command[1] == 'tree':
                    return CompletedProcess(command, 0, 'worth-proof v0.1.0 (proof)\n', '')
                label = 'guarded' if command[command.index('--package') + 1].endswith('guarded') else 'baseline'
                path = Path(command[-1].split('=', 2)[-1])
                path.write_text(f'; {label} independently produced\ndefine i32 @observed_runtime_surface() {{}}\n')
                return CompletedProcess(command, 0, '{"reason":"build-finished","success":true}', '')

            self.assertEqual(prepare(repository, output, runner), output)
            self.assertEqual(len(commands), 4)
            for command in commands:
                if command[1] != 'tree':
                    self.assertIn('--message-format=json', command)
                    self.assertEqual(command[command.index('--target-dir') + 1], str(repository / 'target'))
            self.assertIn('--workspace', commands[0])
            self.assertIn('--keep-going', commands[0])
            self.assertIn('baseline independently', (output / 'baseline.ll').read_text())
            self.assertIn('guarded independently', (output / 'guarded.ll').read_text())
            self.assertEqual((output / 'ready').read_text().strip(), '1')
            self.assertEqual((output / 'runtime-specimens/Cargo.lock').read_text(), 'version = 4\n')

    def test_failed_preparation_invalidates_prior_ready_marker(self):
        with tempfile.TemporaryDirectory() as temporary:
            repository = self.fixture(temporary)
            output = repository / 'target/test-cost/proof'
            output.mkdir(parents=True)
            (output / 'ready').write_text('1\n')
            with self.assertRaises(ProofPrebuildError):
                prepare(repository, output, lambda *_: CompletedProcess([], 101, '', 'setup failed'))
            self.assertFalse((output / 'ready').exists())

    def test_successful_command_without_guarded_ir_cannot_publish_readiness(self):
        with tempfile.TemporaryDirectory() as temporary:
            repository = self.fixture(temporary)
            output = repository / 'target/test-cost/proof'

            def runner(command, *_):
                if command[1] == 'check':
                    return CompletedProcess(command, 101, package_messages(), '')
                if command[1] == 'rustc' and 'zero-cost-baseline' in command:
                    Path(command[-1].split('=', 2)[-1]).write_text('define @observed_runtime_surface')
                return CompletedProcess(command, 0, '', '')

            with self.assertRaisesRegex(ProofPrebuildError, 'guarded release IR was not produced'):
                prepare(repository, output, runner)
            self.assertFalse((output / 'ready').exists())


if __name__ == '__main__':
    unittest.main()
