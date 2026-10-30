import unittest
from pathlib import Path
import tempfile

from executable_cutover import plans, rewrite


class ExecutableCutoverTests(unittest.TestCase):
    def test_only_declared_independent_executable_lookups_gain_refusal_handling(self):
        text = 'let binary = worth_store_test_support::FRESH_PROCESS_RECOVERY.path();\nother.path();'
        changed, count = rewrite(text)
        self.assertEqual(count, 1)
        self.assertIn('.path().expect(', changed)
        self.assertTrue(changed.endswith('other.path();'))

    def test_named_provider_handles_a_multiline_lookup(self):
        changed, count = rewrite('let path = executable\n .path()\n .canonicalize();', True)
        self.assertEqual(count, 1)
        self.assertIn('.path().expect(', changed)

    def test_scoped_lookup_rewrite_preserves_unselected_production_source(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / 'src').mkdir()
            manifest = root / 'Cargo.toml'
            manifest.write_text('[package]\nname="product"\nversion="0.0.0"\n')
            production = 'pub fn production() -> u64 { 42 }\n'
            original = production + '#[test] fn proof() { worth_store_test_support::FRESH_PROCESS_RECOVERY.path(); }'
            (root / 'src/lib.rs').write_text(original)
            planned = list(plans(manifest))
            self.assertEqual(len(planned), 1)
            self.assertEqual(planned[0][1], 1)
            self.assertTrue(planned[0][2].startswith(production))
            self.assertIn('.path().expect(', planned[0][2])
            self.assertEqual((root / 'src/lib.rs').read_text(), original)


if __name__ == '__main__':
    unittest.main()
