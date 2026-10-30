import copy
import hashlib
import json
from pathlib import Path
from subprocess import CompletedProcess
import tempfile
import unittest

from llvm_measure import FLAGS, measure, targets


class FixtureCompiler:
    def __init__(self, rows, environment=None):
        self.environment = environment if environment is not None else {}
        self.rows = rows
        self.calls = []
        self.logs = []
        self.fail_build = False
        self.fail_lines = False
        self.omit_normal = False

    def __call__(self, command, cwd, log):
        self.calls.append((list(command), copy.deepcopy(self.environment)))
        self.logs.append(Path(log))
        if command[:2] == ['cargo', 'llvm-lines']:
            return CompletedProcess(command, int(self.fail_lines), '30 2 TOTAL\n20 2 worth_testkit_world::build<T>\n')
        assert command[:2] == ['cargo', 'test'], command
        assert '--no-run' in command, command
        events = []
        for row in self.rows:
            for harness in ([False, True] if row['family'] == 'lib' else [True]):
                if self.omit_normal and row['family'] == 'lib' and not harness:
                    continue
                stem = row['cargo_target'].replace('-', '_') + '-' + hashlib.sha256(
                    (row['package'] + row['target'] + str(harness)).encode()).hexdigest()[:16]
                deps = Path(cwd) / 'target/debug/deps'
                deps.mkdir(parents=True, exist_ok=True)
                binary = deps / (stem + '.exe' if harness else 'lib' + stem + '.rlib')
                binary.write_bytes(b'linked')
                (deps / (stem + '.ll')).write_bytes(b'define void @build() {}')
                events.append({'reason': 'compiler-artifact', 'package_id': row['package'] + '@0.0.0',
                               'manifest_path': row['manifest'], 'target': {'name': row['cargo_target'],
                               'kind': [row['family']], 'src_path': row['source']},
                               'profile': {'test': harness, 'opt_level': '0'}, 'features': ['product-profile'],
                               'filenames': [str(binary)], 'executable': str(binary) if harness else None,
                               'fresh': False})
        return CompletedProcess(command, int(self.fail_build), '\n'.join(map(json.dumps, events)))


class BulkLlvmMeasurementTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.manifest = self.root / 'Cargo.toml'
        self.manifest.write_text('[workspace]\nmembers=["kit", "court"]\nresolver="2"\n')
        self.package('kit', 'worth-testkit-world', '[lib]\npath="src/build.rs"\n')
        self.package('court', 'worth-court-world', 'autotests=false\n[[test]]\nname="contract"\npath="court/contract.rs"\n',
                     'court/contract.rs')
        self.output = self.root / 'measurement'

    def package(self, directory, name, declaration, source='src/build.rs'):
        root = self.root / directory
        (root / source).parent.mkdir(parents=True, exist_ok=True)
        (root / source).write_text('fn main() {}')
        (root / 'Cargo.toml').write_text('[package]\nname="' + name + '"\nversion="0.0.0"\n' + declaration)

    def test_all_actual_units_share_one_workspace_build_and_only_filemode_follows(self):
        runner = FixtureCompiler(targets(self.manifest))
        measured = measure(self.manifest, self.output, runner, 'ci-test')
        self.assertEqual(set(measured), {'worth-court-world-contract', 'worth-testkit-world-lib',
                                        'worth-testkit-world-lib-test-harness'})
        build = runner.calls[0][0]
        self.assertEqual(build[:5], ['cargo', 'test', '--workspace', '--all-targets', '--no-run'])
        self.assertIn('--message-format=json', build)
        self.assertEqual(build[build.index('--profile') + 1], 'ci-test')
        self.assertIn('build.incremental=false', build)
        self.assertEqual(json.loads(next(value for value in build if value.startswith('build.rustflags=')).split('=', 1)[1]), FLAGS)
        self.assertEqual(sum(command[:2] == ['cargo', 'test'] for command, _ in runner.calls), 1)
        self.assertEqual(len(runner.calls), 4)
        for command, _ in runner.calls[1:]:
            self.assertEqual(command[:3], ['cargo', 'llvm-lines', '--files'])
            self.assertTrue(Path(command[3]).is_file())
        self.assertEqual({row['compiler_log'] for row in measured.values()},
                         {str((self.output / 'workspace-compiler.log').resolve())})
        self.assertEqual(json.loads((self.output / 'measurements.json').read_text()), measured)
        for row in measured.values():
            self.assertEqual(row['profile']['opt_level'], '0')
            self.assertEqual(row['cargo_profile'], 'ci-test')
            self.assertEqual(row['features'], ['product-profile'])
            self.assertEqual(row['rows'][1]['testkit_helpers'], ['worth_testkit_world::build'])
            self.assertEqual(row['compiler_environment']['CARGO_INCREMENTAL'], '0')

    def test_package_filter_changes_attribution_but_never_the_workspace_graph(self):
        runner = FixtureCompiler(targets(self.manifest))
        measured = measure(self.manifest, self.output, runner, selection=['worth-testkit-world'])
        self.assertEqual(len(measured), 2)
        command = runner.calls[0][0]
        self.assertNotIn('-p', command)
        self.assertNotIn('--package', command)
        self.assertIn('--workspace', command)
        self.assertTrue(all(row['package'] == 'worth-testkit-world' for row in measured.values()))

    def test_encoded_flags_and_incremental_override_are_measurement_only(self):
        original = {'CARGO_ENCODED_RUSTFLAGS': '--cfg\x1fname="value with spaces"', 'CARGO_INCREMENTAL': '1'}
        runner = FixtureCompiler(targets(self.manifest), original)
        measure(self.manifest, self.output, runner)
        self.assertIs(runner.environment, original)
        self.assertEqual(runner.calls[0][1]['CARGO_ENCODED_RUSTFLAGS'].split('\x1f'),
                         ['--cfg', 'name="value with spaces"', *FLAGS])
        self.assertEqual(runner.calls[0][1]['CARGO_INCREMENTAL'], '0')
        self.assertEqual(runner.calls[1][1], original)
        self.assertFalse(any('opt-level' in value for value in FLAGS))

    def test_failed_build_or_filemode_restores_the_callers_environment(self):
        for failure in ('fail_build', 'fail_lines'):
            original = {'RUSTFLAGS': '-C overflow-checks=on'}
            runner = FixtureCompiler(targets(self.manifest), original)
            setattr(runner, failure, True)
            with self.subTest(failure=failure), self.assertRaises(ValueError):
                measure(self.manifest, self.output, runner)
            self.assertIs(runner.environment, original)

    def test_harness_cannot_replace_required_normal_library_measurement(self):
        runner = FixtureCompiler(targets(self.manifest))
        runner.omit_normal = True
        with self.assertRaisesRegex(ValueError, 'did not emit required LLVM units: worth-testkit-world-lib'):
            measure(self.manifest, self.output, runner)
        self.assertEqual(len(runner.calls), 1)

    def test_unknown_package_selection_refuses_before_building(self):
        runner = FixtureCompiler(targets(self.manifest))
        with self.assertRaisesRegex(ValueError, 'unknown court/testkit package'):
            measure(self.manifest, self.output, runner, selection=['worth-runtime'])
        self.assertEqual(runner.calls, [])

    def test_manifest_discovery_preserves_explicit_names_paths_and_auto_test_rules(self):
        kit = self.root / 'kit'
        (kit / 'tests').mkdir()
        (kit / 'tests/implicit.rs').write_text('fn main() {}')
        rows = targets(self.manifest)
        self.assertIn(('worth-testkit-world', 'implicit'), {(row['package'], row['target']) for row in rows})
        library = next(row for row in rows if row['family'] == 'lib')
        self.assertEqual(library['source'], str((kit / 'src/build.rs').resolve()))
        self.assertEqual(library['cargo_target'], 'worth_testkit_world')
        (kit / 'Cargo.toml').write_text('[package]\nname="worth-testkit-world"\nversion="0.0.0"\nautotests=false\n'
                                      '[lib]\npath="src/build.rs"\nname="owned_build"\n')
        rows = targets(self.manifest)
        self.assertNotIn('implicit', {row['target'] for row in rows})
        self.assertEqual(next(row for row in rows if row['family'] == 'lib')['cargo_target'], 'owned_build')


if __name__ == '__main__':
    unittest.main()
