from pathlib import Path
import tempfile
import tomllib
import unittest

from compiler_unit_probe import manifests


class BoundedCompilerUnitProbeTests(unittest.TestCase):
    def test_audiences_separate_existing_proof_features_and_reuse_exact_store_profiles_and_lock(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            store = root/'workspaces/worth-store'
            store.mkdir(parents=True)
            (store/'Cargo.toml').write_text('[profile.test]\ndebug=0\n'
                '[profile.diagnostic-test]\ninherits="test"\nopt-level=0\n')
            lock = 'version=4\n[[package]]\nname="sha2"\nversion="0.10.9"\n'
            (store/'Cargo.lock').write_text(lock)
            paths = manifests(root, root/'target/test-cost/unit-probe')
            for audience, path in paths.items():
                value = tomllib.loads(path.read_text())
                proof = value['dependencies']['worth-proof']
                self.assertEqual(proof.get('features', []),
                    ['certification-test-authority'] if audience == 'certification' else [])
                self.assertEqual(value['dependencies']['sha2'], {'version': '=0.10.9'})
                self.assertEqual(value['build-dependencies']['sha2'], '=0.10.9')
                self.assertEqual(value['profile']['diagnostic-test']['opt-level'], 0)
                self.assertEqual(path.with_name('Cargo.lock').read_text(), lock)
                self.assertNotIn('features', value)
                self.assertFalse((path.parent/'target').exists())


if __name__ == '__main__':
    unittest.main()
