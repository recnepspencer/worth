import json
from pathlib import Path
import sys
import tempfile
import unittest

from cargo_capture import CargoRunner, capture_record, instrument


class CargoCaptureTests(unittest.TestCase):
    def test_nested_gate_processes_register_their_complete_evidence_logs(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            index = root / 'index.jsonl'
            capture_record({'WORTH_TEST_COST_CAPTURE_INDEX': str(index)}, root / 'compiler.log')
            self.assertEqual(json.loads(index.read_text())['log'], str((root / 'compiler.log').resolve()))

    def test_compiler_and_nextest_invocations_require_json_without_altering_rustc_flags(self):
        self.assertEqual(instrument(['cargo', 'rustc', '--lib', '--', '--emit=llvm-ir']),
                         ['cargo', 'rustc', '--message-format=json', '--lib', '--', '--emit=llvm-ir'])
        self.assertIn('--cargo-message-format=json', instrument(['cargo', 'nextest', 'list']))
        with self.assertRaises(ValueError):
            instrument(['cargo', 'check', '--message-format=human'])

    def test_noncompiler_metadata_is_preserved_verbatim(self):
        command = ['cargo', 'metadata', '--format-version', '1', '--no-deps']
        self.assertEqual(instrument(command), command)

    def test_prebuilt_nextest_reuse_receives_no_forbidden_cargo_build_flags(self):
        command = ['cargo', 'nextest', 'list', '--cargo-metadata', 'metadata.json',
                   '--binaries-metadata', 'binaries.json']
        self.assertEqual(instrument(command), command)

    def test_expected_denial_returns_raw_output_and_records_nonzero_exit(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            script = root / 'compiler.py'
            script.write_text('import sys\nprint("diagnostic")\nsys.exit(1)\n')
            runner = CargoRunner(Path.cwd())
            result = runner([sys.executable, str(script)], Path.cwd(), root / 'compiler.log')
            self.assertEqual(result.returncode, 1)
            self.assertEqual(result.stdout.strip(), 'diagnostic')
            log = (root / 'compiler.log').read_text()
            self.assertIn('"type": "cargo-invocation"', log)
            self.assertIn('Exit code: 1', log)


if __name__ == '__main__':
    unittest.main()
