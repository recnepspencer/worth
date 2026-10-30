import json
from pathlib import Path
import subprocess
import tempfile
import tomllib
import unittest
from unittest.mock import patch

from compiler_audiences import group
from compiler_prebuild import CompilerScope, normalized_source_key, prepare


class CompilerPrebuildExecutionTests(unittest.TestCase):
    def test_all_and_fixture_scopes_check_two_environments_while_unit_scope_checks_one(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            workspace = root / 'workspaces/worth-store'
            workspace.mkdir(parents=True)
            (workspace / 'Cargo.toml').write_text(
                '[package]\nname="fixture-owner"\nversion="0.1.0"\n'
                '[workspace]\n[profile.diagnostic-test]\ninherits="test"\ndebug=0\n')
            (workspace / 'src').mkdir()
            (workspace / 'src/lib.rs').write_text('pub struct Owner;')
            (workspace / 'Cargo.lock').write_text('version=4\n')
            for name in ('production', 'certification'):
                (root / (name + '.rs')).write_text('fn main() { unavailable::private(); }')

            def declarations(*unused):
                return {name: {'identity': name, 'profile': 'diagnostic-test',
                               'reject_test_features': name == 'production',
                               'packages': group([{'dependencies': {}, 'sources': [root / (name + '.rs')],
                                                   'audience': name}])}
                        for name in ('production', 'certification')}, []

            class Runner:
                def __init__(self):
                    self.environment = {'CARGO_TARGET_DIR': str(workspace / 'target')}
                    self.commands = []

                def __call__(self, command, cwd, log):
                    self.commands.append(command)
                    if command[1] == 'metadata':
                        return subprocess.CompletedProcess(command, 0, json.dumps({
                            'packages': [], 'resolve': {'nodes': []}}), '')
                    manifest = Path(command[command.index('--manifest-path') + 1])
                    document = tomllib.loads(manifest.read_text())
                    events = []
                    for member in document['workspace']['members']:
                        package = tomllib.loads((manifest.parent / member / 'Cargo.toml').read_text())
                        for binary in package['bin']:
                            events.append({'reason': 'compiler-message', 'target': {'name': binary['name']},
                                           'message': {'level': 'error', 'message': 'private fixture denial'}})
                    return subprocess.CompletedProcess(command, 101, '\n'.join(map(json.dumps, events)), '')

            for scope, checks, fixtures in [(CompilerScope.ALL, 2, 5),
                                            (CompilerScope.FIXTURES, 2, 2),
                                            (CompilerScope.UNITS, 1, 3)]:
                with self.subTest(scope=scope):
                    runner = Runner()
                    with patch('compiler_prebuild.environments', side_effect=declarations):
                        environment = prepare(root, 'store', root / scope.value, runner, scope)
                    commands = [command for command in runner.commands if command[1] == 'check']
                    self.assertEqual(len(commands), checks)
                    self.assertTrue(all('--workspace' in command and '--bins' in command and
                                        '--keep-going' in command for command in commands))
                    outcomes = json.loads(Path(environment['WORTH_PREBUILT_COMPILER_OUTCOMES']).read_text())
                    self.assertEqual(len(outcomes['fixtures']), fixtures)
                    self.assertTrue(all(row['audience_identity'] for row in outcomes['fixtures'].values()))
                    if scope is CompilerScope.FIXTURES:
                        self.assertNotIn('WORTH_PREBUILT_COMPILER_UNIT_ROOT', environment)
                        continue
                    unit = Path(environment['WORTH_PREBUILT_COMPILER_UNIT_ROOT'])
                    for name in ('private_a', 'private_b', 'unresolved'):
                        fixture = unit / 'fixtures' / (name + '.rs')
                        self.assertTrue(fixture.is_file())
                        self.assertEqual(outcomes['fixtures'][normalized_source_key(fixture)]['environment'],
                                         'production')
                    if scope is CompilerScope.ALL:
                        production = outcomes['fixtures'][normalized_source_key(root / 'production.rs')]
                        unit_row = outcomes['fixtures'][normalized_source_key(unit / 'fixtures/private_a.rs')]
                        self.assertEqual(production['environment_identity'], unit_row['environment_identity'])
                        self.assertNotEqual(production['audience_identity'], unit_row['audience_identity'])
                        self.assertFalse(any('compiler-unit/Cargo.toml' in ' '.join(command).replace('\\', '/')
                                             for command in commands))


if __name__ == '__main__':
    unittest.main()
