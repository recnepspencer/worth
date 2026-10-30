from pathlib import Path
import json
import tempfile
import unittest

from llvm_measure import measurement_name, publish, targets
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

    def test_ir_publication_preserves_native_and_doc_execution_evidence(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            census = {'manifest': 'Cargo.toml', 'source_counts': {}, 'source_sites': [],
                      'lanes': {}, 'coverage': {'status': 'pass'}, 'llvm_lines': {},
                      'executed_coverage': {'passed': 42}, 'documentation': {'status': 'pass'}}
            (root / 'census.json').write_text(json.dumps(census))
            measured = {'court': {'profile': 'test', 'compiler_log': 'retained.log',
                                  'rows': llvm_lines('30 2 TOTAL\n20 2 worth_testkit_world::build<T>')}}
            publish(root, measured)
            actual = json.loads((root / 'census.json').read_text())
            self.assertEqual(actual['executed_coverage'], census['executed_coverage'])
            self.assertEqual(actual['documentation'], census['documentation'])
            self.assertEqual(actual['llvm_measurements']['court']['compiler_log'], 'retained.log')
            self.assertIn('worth_testkit_world::build', (root / 'census.md').read_text())


if __name__ == '__main__':
    unittest.main()
