import json
from pathlib import Path
import subprocess
import tempfile
import unittest
import re
import tomllib

from nextest_build import LANES, NextestPrebuildFailure, build, list_tests, summary, selection_arguments, source_packages
from workspace import members, repository_root


class PrebuiltNextestTests(unittest.TestCase):
    def test_committed_store_binary_selectors_name_real_court_targets(self):
        root = repository_root(Path(__file__).parent)
        manifest = root / 'workspaces/worth-store/Cargo.toml'
        real = set()
        for package in members(manifest):
            data = tomllib.loads(package['manifest'].read_text(encoding='utf-8'))
            real.update(package['name'] + '::' + target['name'] for target in data.get('test', []))
        config = tomllib.loads((manifest.parent / '.config/nextest.toml').read_text())
        for profile in config['profile'].values():
            for override in profile.get('overrides', []):
                for binary in re.findall(r'binary_id\(=([^)]*)\)', override['filter']):
                    self.assertIn(binary, real)
        self.assertNotIn('compiler-boundary', config.get('test-groups', {}))

    def test_pretty_inventory_is_separated_from_compiler_jsonl(self):
        text = '{"reason":"compiler-artifact"}\n' + json.dumps({'rust-binaries': {'court': {}}}, indent=2)
        self.assertEqual(summary(text, 'rust-binaries'), {'rust-binaries': {'court': {}}})

    def test_build_is_once_and_listing_uses_only_prebuilt_metadata(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            commands = []

            def runner(command, cwd, log):
                commands.append(command)
                key = 'rust-binaries' if 'binaries-only' in command else 'rust-suites'
                return subprocess.CompletedProcess(command, 0, json.dumps({key: {}}), '')

            reuse = build(root / 'Cargo.toml', root / 'out', runner)
            list_tests(reuse, root / 'out/inventory.json', runner, partition='hash:1/2')
            self.assertEqual(len(commands), 3)
            self.assertIn('--binaries-metadata', commands[-1])
            self.assertIn('--ignore-default-filter', commands[-1])
            self.assertEqual(commands[-1][-2:], ['--partition', 'hash:1/2'])
            self.assertNotIn('--manifest-path', commands[-1])
            self.assertNotIn('--all-targets', commands[-1])
            self.assertNotIn('--include-ignored', ' '.join(' '.join(c) for c in commands))

    def test_category_lanes_use_paths_and_ordinary_excludes_them(self):
        self.assertIn('test(/^compile_fail::/)', LANES['compile-fail'])
        self.assertIn('not ', LANES['ordinary'])

    def test_owner_selection_is_resolved_before_the_only_compiling_command(self):
        metadata = {'workspace_members': ['owner', 'other'], 'packages': [
            {'id': name, 'name': name, 'targets': [{'name': name, 'kind': ['lib']}]}
            for name in ('owner', 'other')]}
        with tempfile.TemporaryDirectory() as temporary:
            root, commands = Path(temporary), []
            def runner(command, cwd, log):
                commands.append(command)
                value = metadata if command[1] == 'metadata' else {'rust-binaries': {}}
                return subprocess.CompletedProcess(command, 0, json.dumps(value), '')
            def selection(observed):
                self.assertEqual(observed, metadata)
                self.assertEqual(len(commands), 1)
                return {'packages': ['owner'], 'targets': {'lib': True}}
            build(root/'Cargo.toml', root/'out', runner, 'dev', selection)
            command = commands[1]
            self.assertEqual(command[3:7], ['--package', 'owner', '--lib', '--cargo-profile'])
            self.assertNotIn('--workspace', command)
            self.assertNotIn('--all-targets', command)
            self.assertEqual(command[command.index('--cargo-profile') + 1], 'dev')
            self.assertNotIn('--features', command)

    def test_invalid_owners_and_targets_are_refused_without_fallback_to_workspace(self):
        metadata = {'workspace_members': ['owner'], 'packages': [
            {'id': 'owner', 'name': 'owner', 'targets': [{'name': 'claims', 'kind': ['test']}]}]}
        for selection in ({'packages': []}, {'packages': ['absent']},
                          {'packages': ['owner'], 'targets': {'lib': True}},
                          {'packages': ['owner'], 'targets': {'test': ['absent']}},
                          {'packages': ['owner'], 'targets': {'unknown': ['claims']}}):
            with self.subTest(selection=selection), self.assertRaises(NextestPrebuildFailure):
                selection_arguments(metadata, selection)
        self.assertEqual(selection_arguments(metadata, {'packages': ['owner'], 'targets': {'test': ['claims']}}),
                         ['--package', 'owner', '--test', 'claims'])

    def test_source_scope_includes_resolved_transitive_testkits_but_not_unrelated_courts(self):
        names = ['owner', 'testkit', 'dependency', 'unrelated-court']
        metadata = {'workspace_members': names, 'packages': [{'id': name, 'name': name} for name in names],
                    'resolve': {'nodes': [
                        {'id': 'owner', 'dependencies': ['testkit']},
                        {'id': 'testkit', 'dependencies': ['dependency']},
                        {'id': 'dependency', 'dependencies': []},
                        {'id': 'unrelated-court', 'dependencies': []}]}}
        self.assertEqual(source_packages(metadata, {'packages': ['owner']}), ['dependency', 'owner', 'testkit'])
        self.assertIsNone(source_packages(metadata, None))
        metadata['resolve']['nodes'].pop(1)
        with self.assertRaisesRegex(NextestPrebuildFailure, 'resolved dependency'):
            source_packages(metadata, {'packages': ['owner']})

    def test_source_boundary_refusal_happens_after_metadata_and_before_compilation(self):
        with tempfile.TemporaryDirectory() as temporary:
            commands = []
            def runner(command, cwd, log):
                commands.append(command)
                return subprocess.CompletedProcess(command, 0, '{}', '')
            def refuse(metadata, selection):
                self.assertEqual(len(commands), 1)
                raise ValueError('tests invoke compilers')
            with self.assertRaisesRegex(ValueError, 'tests invoke compilers'):
                build(Path(temporary)/'Cargo.toml', Path(temporary)/'out', runner, before_compile=refuse)
            self.assertEqual(commands[0][1], 'metadata')


if __name__ == '__main__':
    unittest.main()
