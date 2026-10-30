from pathlib import Path
import tempfile
import unittest

from compiler_features import is_test_feature, violations, vocabulary
from workspace import repository_root


class ProductionCompilerFeatureTests(unittest.TestCase):
    def test_test_support_is_refused_without_any_certification_feature(self):
        with tempfile.TemporaryDirectory() as temporary:
            manifest = Path(temporary) / 'Cargo.toml'
            manifest.write_text('[workspace]\n[package]\nname="owner"\nversion="0.1.0"\n'
                                '[features]\ntest-support=[]\nrecovery-runtime-fixtures=[]\n')
            metadata = {'resolve': {'nodes': [{'id': 'owner', 'features': ['test-support']}]}}
            self.assertEqual(violations(manifest, metadata), [('owner', 'test-support')])
            metadata['resolve']['nodes'][0]['features'] = ['recovery-runtime-fixtures']
            self.assertEqual(violations(manifest, metadata), [('owner', 'recovery-runtime-fixtures')])

    def test_existing_test_flags_are_distinct_from_owner_authority_and_product_profiles(self):
        declared = {'recovery-runtime-fixtures', 'physical-isolation-fixtures'}
        for feature in ['test-support', 'test-peak-allocation', 'volatile-sync-for-tests',
                        'certification-authority', 'recovery-runtime-fixtures', 'physical-isolation-fixtures']:
            self.assertTrue(is_test_feature(feature, declared), feature)
        for feature in ['store-runtime-owner', 'recovery-runtime-owner', 'profile-extended',
                        'default', 'wasm-bindgen']:
            self.assertFalse(is_test_feature(feature, declared), feature)

    def test_store_vocabulary_includes_root_signal_test_flags_and_owned_fixture_flags(self):
        root = repository_root(Path(__file__).parent)
        names = vocabulary(root / 'workspaces/worth-store/Cargo.toml')
        self.assertIn('recovery-runtime-fixtures', names)
        self.assertIn('test-peak-allocation', names)
        self.assertIn('profile-extended', names)
        self.assertFalse(is_test_feature('profile-extended', names))

    def test_independent_workspace_does_not_require_an_unrelated_parent_cargo_root(self):
        with tempfile.TemporaryDirectory() as temporary:
            manifest = Path(temporary) / 'workspaces/standalone/Cargo.toml'
            manifest.parent.mkdir(parents=True)
            manifest.write_text('[package]\nname="owner"\nversion="0.1.0"\n'
                                '[workspace]\n[features]\nowned-fixtures=[]\n')
            self.assertEqual(vocabulary(manifest), {'owned-fixtures'})


if __name__ == '__main__':
    unittest.main()
