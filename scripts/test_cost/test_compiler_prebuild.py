from pathlib import Path
import tempfile
import unittest

from compiler_audiences import group, sources
from compiler_prebuild import environments, normalized_source_key, recipe_identity, unit_inputs
from workspace import repository_root


class CompilerPrebuildInputTests(unittest.TestCase):
    def test_unit_world_preserves_real_private_and_wrong_reason_compiler_inputs(self):
        with tempfile.TemporaryDirectory() as temporary:
            root, environment = unit_inputs(temporary)
            self.assertEqual(len(sources(environment)), 3)
            self.assertEqual(environment['environment_class'], 'production')
            self.assertTrue(environment['reject_test_features'])
            self.assertIn('sealed_dependency::private()', (root / 'fixtures/private_a.rs').read_text())
            self.assertIn('absent_dependency::private', (root / 'fixtures/unresolved.rs').read_text())
            self.assertIn('OPT_LEVEL', (root / 'dependency/build.rs').read_text())

    def test_source_key_is_a_concrete_canonical_path(self):
        path = normalized_source_key(Path(__file__))
        self.assertNotIn('\\', path)
        self.assertTrue(path.endswith('/test_compiler_prebuild.py'))

    def test_recipe_identity_covers_every_generated_package_manifest(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            manifest = root / 'Cargo.toml'
            manifest.write_text('[workspace]\n[profile.test]\ndebug=0\n')
            row = dict(identity='production', profile='test', packages=group([
                {'dependencies': {}, 'sources': [root / 'one.rs']},
                {'dependencies': {'backend': {'version': '1'}}, 'sources': [root / 'two.rs']}]))
            original = recipe_identity(root, manifest, row)
            row['packages'] = group([
                {'dependencies': {}, 'sources': [root / 'one.rs']},
                {'dependencies': {'backend': {'version': '2'}}, 'sources': [root / 'two.rs']}])
            self.assertNotEqual(original, recipe_identity(root, manifest, row))
            row['packages'] = group([
                {'dependencies': {}, 'sources': [root / 'one.rs']},
                {'dependencies': {'backend': {'version': '1'}},
                 'target_dependencies': {'cfg(windows)': {'native': {'version': '1'}}},
                 'sources': [root / 'two.rs']}])
            self.assertNotEqual(original, recipe_identity(root, manifest, row))

    def test_real_recovery_fixture_keeps_format_suggestion_without_backend_import_authority(self):
        root = repository_root(Path(__file__).parent)
        rows, orphaned = environments(root, 'store')
        self.assertEqual(orphaned, [])
        selected = [package for package in rows['certification']['packages']
                    if any(source.name == 'ordinary_backend_qualification_is_not_a_recovery_dependency.rs'
                           for source in package['sources'])]
        self.assertEqual(len(selected), 1)
        self.assertIn('worth-store-physical-format', selected[0]['dependencies'])
        self.assertNotIn('worth-store-physical-backend', selected[0]['dependencies'])
        self.assertNotIn('worth-store-testkit-recovery-runtime', selected[0]['dependencies'])
        absence = [package for package in rows['production']['packages']
                   if any(source.name == 'certification_authority_is_absent.rs' for source in package['sources'])]
        self.assertEqual(len(absence), 1)
        self.assertEqual(set(absence[0]['dependencies']), {'worth-store'})
        self.assertNotIn('features', absence[0]['dependencies']['worth-store'])
        self.assertTrue(rows['production']['reject_test_features'])

    def test_production_fixture_graphs_do_not_compile_a_test_harness(self):
        root = repository_root(Path(__file__).parent)
        for workspace in ('root', 'query', 'bank', 'store'):
            with self.subTest(workspace=workspace):
                rows, _ = environments(root, workspace)
                self.assertNotIn(rows['production']['profile'], ('test', 'bench'))
        self.assertEqual(environments(root, 'root')[0]['production']['profile'], 'dev')

    def test_every_governed_production_fixture_environment_audits_test_features(self):
        root = repository_root(Path(__file__).parent)
        for workspace in ('root', 'query', 'bank', 'store'):
            with self.subTest(workspace=workspace):
                rows, _ = environments(root, workspace)
                self.assertTrue(rows['production']['reject_test_features'])

    def test_nullable_optional_is_codegen_only_in_the_same_normal_production_audience(self):
        rows, _ = environments(repository_root(Path(__file__).parent), 'root')
        generated = rows['production-codegen']
        self.assertEqual([source.name for source in sources(generated)], ['nullable_optional.rs'])
        self.assertEqual(generated['proof_stage'], 'codegen')
        self.assertEqual(generated['environment_class'], 'production')
        self.assertEqual(generated['profile'], 'dev')
        self.assertTrue(generated['reject_test_features'])
        self.assertNotIn('nullable_optional.rs', [source.name for source in sources(rows['production'])])


if __name__ == '__main__':
    unittest.main()
