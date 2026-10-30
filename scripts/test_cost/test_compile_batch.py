import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from compile_batch import BatchCompilationFailure, attribute, bin_name, render_manifests, verify_locked_dependencies, target_directory, prepare_environment
from compiler_audiences import group, sources


def target(source, kind='bin', name=None):
    return {'name': name or bin_name(source), 'kind': [kind], 'src_path': str(Path(source).resolve())}


class BatchedCompilerTests(unittest.TestCase):
    def test_test_harness_profiles_are_refused_before_any_compiler_or_output(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            output = root / 'output'
            for profile in ('test', 'bench'):
                with self.subTest(profile=profile), self.assertRaisesRegex(
                        BatchCompilationFailure, 'require normal binaries'):
                    prepare_environment(root / 'Cargo.toml', {'profile': profile}, output,
                                        lambda *_: self.fail('compiler called for a test harness'))
                self.assertFalse(output.exists())

    def test_gate_owned_target_directory_is_used_without_creating_a_fixture_target(self):
        from types import SimpleNamespace
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            runner = SimpleNamespace(environment={'CARGO_TARGET_DIR': str(root / 'active-target')})
            self.assertEqual(target_directory(root / 'fixture/Cargo.toml', runner), root / 'active-target')

    def test_generated_package_may_be_new_but_registry_versions_stay_workspace_locked(self):
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / 'Cargo.lock'
            path.write_text('[[package]]\nname="serde"\nversion="1.0.0"\nsource="registry+locked"\n')
            allowed = dict(name='serde', version='1.0.0', source='registry+locked')
            verify_locked_dependencies(path, {'packages': [allowed, dict(name='generated', version='0.0.0', source=None)]})
            with self.assertRaises(BatchCompilationFailure):
                verify_locked_dependencies(path, {'packages': [dict(allowed, version='1.0.1')]})

    def test_each_diagnostic_is_attributed_to_its_fixture_not_the_aggregate_exit(self):
        sources = [str(Path('one.rs').resolve()), str(Path('two.rs').resolve())]
        events = [dict(reason='compiler-message', target=target(sources[0]),
                       message={'level': 'error', 'message': 'private field'}),
                  dict(reason='compiler-artifact', target=target(sources[1]))]
        result = attribute(events, sources)
        self.assertEqual([result[s]['fate'] for s in sources], ['denied', 'compiled'])
        self.assertEqual(result[sources[1]]['messages'], [])

    def test_uncompiled_fixture_or_dependency_failure_cannot_satisfy_denial(self):
        with self.assertRaises(BatchCompilationFailure):
            attribute([], ['not_attempted.rs'])
        with self.assertRaises(BatchCompilationFailure):
            attribute([dict(reason='compiler-message', target=target('dependency.rs', 'lib', 'dependency'),
                            message={'level': 'error'})], [])

    def test_workspace_preserves_profiles_and_separates_direct_dependency_audiences(self):
        with tempfile.TemporaryDirectory() as temporary:
            manifest = Path(temporary) / 'Cargo.toml'
            manifest.write_text('[profile.test]\nopt-level=0\n[profile.test.package."*"]\nopt-level=2\n'
                                '[profile.diagnostic-test]\ninherits="test"\ndebug=0\n')
            env = dict(identity='production', packages=group([
                {'dependencies': {}, 'target_dependencies': {'cfg(windows)': {'native': {'version': '1'}}},
                 'sources': ['one.rs']},
                {'dependencies': {'backend': {'version': '1'}}, 'sources': ['two.rs']}]))
            import tomllib
            manifests = render_manifests(env, manifest)
            result = tomllib.loads(manifests['Cargo.toml'])
            self.assertNotIn('package', result)
            self.assertEqual(len(result['workspace']['members']), 2)
            self.assertEqual(result['profile']['test']['package']['*']['opt-level'], 2)
            self.assertEqual(result['profile']['diagnostic-test']['inherits'], 'test')
            packages = [tomllib.loads(text) for name, text in manifests.items() if name != 'Cargo.toml']
            self.assertTrue(all(len(package['package']['name']) <= 64 for package in packages))
            self.assertEqual(sum('backend' in package['dependencies'] for package in packages), 1)
            self.assertEqual(sum(len(package['bin']) for package in packages), 2)
            self.assertTrue(all('native' not in package['dependencies'] for package in packages))
            self.assertEqual(sum('cfg(windows)' in package.get('target', {}) for package in packages), 1)

    def test_two_audiences_share_one_check_and_every_outcome_retains_its_surface_identity(self):
        import subprocess
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            manifest = root / 'Cargo.toml'
            manifest.write_text('[package]\nname="fixture-owner"\nversion="0.1.0"\n'
                                '[workspace]\n[profile.test]\ndebug=0\n')
            (root / 'src').mkdir()
            (root / 'src/lib.rs').write_text('pub struct Owner;')
            lock = root / 'Cargo.lock'
            lock.write_text('version=4\n')
            env = dict(identity='production', profile='dev', reject_test_features=True, packages=group([
                {'dependencies': {'format': {'version': '1'}}, 'sources': [root / 'forbidden.rs']},
                {'dependencies': {'format': {'version': '1'}, 'backend': {'version': '1'}},
                 'sources': [root / 'allowed.rs']}]))

            class Runner:
                environment = {'CARGO_TARGET_DIR': str(root / 'active-target')}
                commands = []

                def __call__(self, command, cwd, log):
                    self.commands.append(command)
                    if command[1] == 'metadata':
                        return subprocess.CompletedProcess(command, 0, json.dumps({
                            'packages': [], 'resolve': {'nodes': []}}), '')
                    events = [dict(reason='compiler-message', target=target(root / 'forbidden.rs'),
                                   message={'level': 'error', 'message': 'unlinked backend',
                                            'code': {'code': 'E0432'}}),
                              dict(reason='compiler-artifact', target=target(root / 'allowed.rs'))]
                    return subprocess.CompletedProcess(command, 101, '\n'.join(map(json.dumps, events)), '')

            runner = Runner()
            result = prepare_environment(manifest, env, root / 'batch', runner)
            checks = [command for command in runner.commands if command[1] == 'check']
            self.assertEqual(len(checks), 1)
            for flag in ('--workspace', '--bins', '--keep-going', '--locked'):
                self.assertIn(flag, checks[0])
            self.assertEqual((root / 'batch/Cargo.lock').read_text(), lock.read_text())
            outcomes = result['fixtures']
            self.assertEqual(outcomes[str(root / 'forbidden.rs')]['fate'], 'denied')
            self.assertEqual(outcomes[str(root / 'allowed.rs')]['fate'], 'compiled')
            self.assertEqual({row['audience_identity'] for row in outcomes.values()},
                             {package['identity'] for package in env['packages']})

    def test_duplicate_source_in_multiple_packages_is_refused_before_manifest_creation(self):
        packages = group([{'dependencies': {}, 'sources': ['one.rs']},
                          {'dependencies': {'backend': {'version': '1'}}, 'sources': ['two.rs']}])
        packages[1]['sources'] = list(packages[0]['sources'])
        with self.assertRaisesRegex(BatchCompilationFailure, 'duplicate fixture source'):
            render_manifests({'identity': 'production', 'packages': packages}, Path('unused.toml'))

    def test_production_feature_audit_refuses_authority_from_any_workspace_package(self):
        import subprocess
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            manifest = root / 'Cargo.toml'
            manifest.write_text('[package]\nname="fixture-owner"\nversion="0.1.0"\n[workspace]\n')
            (root / 'src').mkdir()
            (root / 'src/lib.rs').write_text('pub struct Owner;')
            (root / 'Cargo.lock').write_text('version=4\n')
            environment = dict(identity='production', profile='dev', reject_test_features=True,
                               packages=group([{'dependencies': {}, 'sources': [root / 'one.rs']}]))
            for feature in ('certification-authority', 'test-support'):
                with self.subTest(feature=feature):
                    commands = []

                    def runner(command, cwd, log):
                        commands.append(command)
                        return subprocess.CompletedProcess(command, 0, json.dumps({
                            'packages': [], 'resolve': {'nodes': [
                                {'id': 'another-audience-dependency', 'features': [feature]}]}}), '')

                    with self.assertRaisesRegex(BatchCompilationFailure, 'production compiler graph enables'):
                        prepare_environment(manifest, environment, root / 'batch', runner)
                    self.assertFalse(any(command[1] == 'check' for command in commands))

    def test_same_target_name_from_another_source_is_still_a_dependency_error(self):
        source = Path('fixture.rs').resolve()
        forged = dict(reason='compiler-message',
                      target=target(source.parent / 'dependency/fixture.rs', name=bin_name(source)),
                      message={'level': 'error', 'message': 'dependency error'})
        genuine = dict(reason='compiler-message', target=target(source),
                       message={'level': 'error', 'message': 'fixture denial'})
        for events in ([forged], [forged, genuine]):
            with self.assertRaisesRegex(BatchCompilationFailure, 'fixture dependency failed'):
                attribute(events, [source])

    def test_matching_source_with_a_library_target_is_a_dependency_error(self):
        source = Path('fixture.rs').resolve()
        event = dict(reason='compiler-message', target=target(source, 'lib'),
                     message={'level': 'error', 'message': 'wrong target kind'})
        with self.assertRaisesRegex(BatchCompilationFailure, 'fixture dependency failed'):
            attribute([event], [source])

    def test_dependency_artifacts_cannot_prove_fixture_compilation_or_enter_its_evidence(self):
        source = Path('fixture.rs').resolve()
        denied = dict(reason='compiler-message', target=target(source),
                      message={'level': 'error', 'message': 'fixture denial'})
        for metadata in [target(source.parent / 'dependency/fixture.rs', name=bin_name(source)),
                         target(source, 'lib')]:
            artifact = dict(reason='compiler-artifact', target=metadata)
            with self.assertRaisesRegex(BatchCompilationFailure, 'did not compile every'):
                attribute([artifact], [source])
            outcome = attribute([artifact, denied], [source])[str(source)]
            self.assertEqual(outcome['fate'], 'denied')
            self.assertEqual(outcome['artifacts'], [])

    def test_missing_or_malformed_target_metadata_is_refused(self):
        source = Path('fixture.rs').resolve()
        valid = target(source)
        invalid = [None, {}, {'name': valid['name']}, dict(valid, kind='bin'), dict(valid, kind=[]),
                   dict(valid, kind=['unknown-kind']), dict(valid, src_path=None),
                   dict(valid, src_path='relative.rs'), dict(valid, name=7)]
        for reason in ('compiler-message', 'compiler-artifact'):
            for metadata in invalid:
                with self.subTest(reason=reason, target=metadata):
                    with self.assertRaisesRegex(BatchCompilationFailure, 'malformed target metadata'):
                        attribute([dict(reason=reason, target=metadata, message={'level': 'error'})], [source])

    def test_different_declared_sources_cannot_share_a_generated_target_name(self):
        with patch('compile_batch.bin_name', return_value='fixture_collision'):
            with self.assertRaisesRegex(BatchCompilationFailure, 'target name collision'):
                attribute([], ['one.rs', 'two.rs'])
            environment = {'identity': 'production', 'packages': group([
                {'dependencies': {}, 'sources': ['one.rs', 'two.rs']}])}
            with self.assertRaisesRegex(BatchCompilationFailure, 'target name collision'):
                render_manifests(environment, Path('unused.toml'))

    def test_fixture_target_keeps_diagnostics_whose_spans_point_into_a_support_module(self):
        source = Path('fixture.rs').resolve()
        message = {'level': 'error', 'message': 'private support item',
                   'spans': [{'file_name': str(source.parent / 'support/module.rs'), 'is_primary': True}]}
        result = attribute([dict(reason='compiler-message', target=target(source), message=message)], [source])
        self.assertEqual(result[str(source)]['fate'], 'denied')
        self.assertEqual(result[str(source)]['messages'], [message])


if __name__ == '__main__':
    unittest.main()
