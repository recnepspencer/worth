from pathlib import Path
import tempfile
import unittest

from llvm_measure import measurement_name, targets
from logs import llvm_lines


class CompileSizeOwnershipTests(unittest.TestCase):
    def test_long_court_names_have_bounded_distinct_measurement_filenames(self):
        prefix = 'worth-store-court-' + 'production-frontier-' * 15
        first, second = measurement_name(prefix + 'one'), measurement_name(prefix + 'two')
        self.assertLessEqual(len(first), 80)
        self.assertNotEqual(first, second)
        self.assertEqual(first, measurement_name(prefix + 'one'))

    def test_only_real_court_and_testkit_targets_enter_compile_size_measurement(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / 'src').mkdir()
            (root / 'src/lib.rs').write_text('pub fn build() {}')
            manifest = root / 'Cargo.toml'
            manifest.write_text('[package]\nname="worth-testkit-world"\nversion="0.0.0"\n')
            self.assertEqual(targets(manifest)[0]['selector'], ['--lib'])
            manifest.write_text('[package]\nname="worth-runtime"\nversion="0.0.0"\n')
            self.assertEqual(targets(manifest), [])

    def test_generic_instantiations_keep_the_real_testkit_function_path(self):
        rows = llvm_lines('30 2 TOTAL\n20 2 worth_store_testkit_world::build<T>')
        self.assertEqual(rows[1]['testkit_helpers'], ['worth_store_testkit_world::build'])


if __name__ == '__main__':
    unittest.main()
