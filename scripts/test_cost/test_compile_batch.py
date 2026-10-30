import json
from pathlib import Path
import tempfile
import unittest

from compile_batch import BatchCompilationFailure, attribute, bin_name, render_manifest, verify_locked_dependencies, target_directory


class BatchedCompilerTests(unittest.TestCase):
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
        events = [dict(reason='compiler-message', target={'name': bin_name(sources[0])},
                       message={'level': 'error', 'message': 'private field'}),
                  dict(reason='compiler-artifact', target={'name': bin_name(sources[1])})]
        result = attribute(events, sources)
        self.assertEqual([result[s]['fate'] for s in sources], ['denied', 'compiled'])
        self.assertEqual(result[sources[1]]['messages'], [])

    def test_uncompiled_fixture_or_dependency_failure_cannot_satisfy_denial(self):
        with self.assertRaises(BatchCompilationFailure):
            attribute([], ['not_attempted.rs'])
        with self.assertRaises(BatchCompilationFailure):
            attribute([dict(reason='compiler-message', target={'name': 'dependency'},
                            message={'level': 'error'})], [])

    def test_one_manifest_contains_all_bins_and_preserves_workspace_profiles(self):
        with tempfile.TemporaryDirectory() as temporary:
            manifest = Path(temporary) / 'Cargo.toml'
            manifest.write_text('[profile.test]\nopt-level=0\n[profile.test.package."*"]\nopt-level=2\n'
                                '[profile.diagnostic-test]\ninherits="test"\ndebug=0\n')
            env = dict(identity='production', dependency_manifest='[dependencies]\n',
                       sources=['one.rs', 'two.rs'])
            import tomllib
            result = tomllib.loads(render_manifest(env, manifest))
            self.assertEqual(len(result['bin']), 2)
            self.assertEqual(result['profile']['test']['package']['*']['opt-level'], 2)
            self.assertEqual(result['profile']['diagnostic-test']['inherits'], 'test')


if __name__ == '__main__':
    unittest.main()
