import copy
import hashlib
from pathlib import Path
import tempfile
import unittest

from llvm_measure import targets
from llvm_probe import materialize, validate


class LlvmProtocolProbeTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)

    def measured(self):
        library = self.root / 'libprobe.rlib'
        executable = self.root / 'probe.exe'
        library.write_bytes(b'!<arch>\nfixture archive')
        executable.write_bytes(b'MZfixture executable')
        ir = {'bytes': 32, 'sha256': hashlib.sha256(b'fixture IR').hexdigest(), 'path': 'exact.ll'}
        rows = [{'name': 'TOTAL', 'lines': 20, 'testkit_helpers': []},
                {'name': 'worth_testkit_ir_probe::build', 'lines': 5,
                 'testkit_helpers': ['worth_testkit_ir_probe::build']}]
        normal = dict(rows=rows, executable=None, filenames=[str(library)], ir=ir)
        harness = dict(rows=rows, executable=str(executable), filenames=[str(executable)], ir=ir)
        return {'worth-testkit-ir-probe-lib': normal,
                'worth-testkit-ir-probe-lib-test-harness': harness,
                'worth-court-ir-probe-ir_contract': copy.deepcopy(harness)}

    def test_probe_declares_real_library_and_integration_tests_that_panic_if_executed(self):
        manifest = materialize(self.root)
        rows = targets(manifest)
        self.assertEqual({(row['package'], row['family']) for row in rows},
                         {('worth-testkit-ir-probe', 'lib'), ('worth-court-ir-probe', 'test')})
        for path in (self.root / 'kit/src/lib.rs', self.root / 'court/tests/ir_contract.rs'):
            text = path.read_text()
            self.assertIn('#[test]', text)
            self.assertIn('panic!("LLVM probe', text)
            self.assertNotIn('#[ignore]', text)

    def test_probe_requires_actual_totals_helpers_and_retained_linked_outputs(self):
        result = validate(self.measured())
        self.assertEqual(result['tests_executed'], 0)
        self.assertEqual(len(result['llvm_ir_lines']), 3)
        self.assertIn('worth_testkit_ir_probe::build', result['testkit_helpers'])
        for failure in ('missing-unit', 'no-helper', 'empty-ir', 'invalid-archive', 'invalid-executable'):
            measured = self.measured()
            if failure == 'missing-unit':
                del measured['worth-testkit-ir-probe-lib']
            elif failure == 'no-helper':
                for row in measured.values():
                    for entry in row['rows']:
                        entry['testkit_helpers'] = []
            elif failure == 'empty-ir':
                measured['worth-testkit-ir-probe-lib']['ir']['bytes'] = 0
            elif failure == 'invalid-archive':
                (self.root / 'libprobe.rlib').write_bytes(b'not linked')
            else:
                (self.root / 'probe.exe').write_bytes(b'')
            with self.subTest(failure=failure), self.assertRaises(ValueError):
                validate(measured)


if __name__ == '__main__':
    unittest.main()
