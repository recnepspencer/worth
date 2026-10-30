import json
from pathlib import Path
import sys
import tempfile
import unittest
import subprocess
from unittest.mock import patch

from cargo_capture import CargoRunner, capture_record, compiler_profile, instrument, retention_name
from test_compiler_units import graph, unit
from compiler_units import CompilerUnitRefusal
from unit_plan import declare


class CargoCaptureTests(unittest.TestCase):
    def test_runtime_nextest_profiles_cannot_disguise_the_same_compiler_graph(self):
        self.assertEqual(compiler_profile(['cargo', 'nextest', 'run', '--profile', 'ci']), 'test')
        self.assertEqual(compiler_profile(['cargo', 'nextest', 'run', '--profile', 'integration',
                                           '--cargo-profile=ci-test']), 'ci-test')
        self.assertEqual(compiler_profile(['cargo', 'check', '--profile=diagnostic-test']), 'diagnostic-test')
        self.assertEqual(compiler_profile(['cargo', 'rustc', '--release', '--', '--profile=not-cargo']), 'release')
        self.assertIsNone(compiler_profile(['python', 'phase_telemetry.py']))

    def test_doctest_owner_headings_keep_their_actual_order_with_result_blocks(self):
        with tempfile.TemporaryDirectory() as temporary:
            runner = CargoRunner(Path.cwd())
            stream = 'Doc-tests owner\nrunning 0 tests\ntest result: ok. 0 passed; 0 failed; 0 ignored\n'
            def execute(command, **kwargs):
                output = json.dumps(graph(unit('owner'))) if '--unit-graph' in command else stream
                return subprocess.CompletedProcess(command, 0, output, None)
            with patch('cargo_capture.subprocess.run', side_effect=execute) as child:
                observed = runner(['cargo', 'test', '--doc'], Path.cwd(), Path(temporary) / 'doc.log')
            self.assertEqual(child.call_args.kwargs['stderr'], subprocess.STDOUT)
            self.assertEqual(observed.stdout, stream)
            self.assertEqual(observed.stderr, '')

    def test_bootstrap_applies_only_to_noncompiling_declaration_and_actual_plan_precedes_build(self):
        with tempfile.TemporaryDirectory() as temporary:
            calls = []
            environment = {'RUSTFLAGS': '-C overflow-checks=on'}
            runner = CargoRunner(Path.cwd(), environment)

            def execute(command, **kwargs):
                if command[:1] == ['cargo']:
                    calls.append((command, dict(kwargs['env'])))
                output = json.dumps(graph(unit('owner'))) if '--unit-graph' in command else ''
                return subprocess.CompletedProcess(command, 0, output, '')

            with patch('cargo_capture.subprocess.run', side_effect=execute):
                runner(['cargo', 'check'], Path.cwd(), Path(temporary) / 'check.log')
            self.assertEqual(len(calls), 2)
            self.assertIn('--unit-graph', calls[0][0])
            self.assertEqual(calls[0][1]['RUSTC_BOOTSTRAP'], '1')
            self.assertNotIn('RUSTC_BOOTSTRAP', calls[1][1])
            self.assertIs(runner.environment, environment)
            log = (Path(temporary) / 'check.log').read_text()
            self.assertIn('"unit_plan":', log)
            self.assertTrue((Path(temporary) / 'check-units.json').is_file())

    def test_preexecution_drift_refusal_retains_footer_without_invoking_the_compiler(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            command = ['cargo', 'check', '--message-format=json']
            plan = declare(command, root, {}, json.dumps(graph(unit('owner'))))
            plan['graph_log'] = 'declared.log'
            runner = CargoRunner(Path.cwd(), {'RUSTFLAGS': 'unplanned'})
            with patch.object(runner, 'unit_plan', return_value=plan), \
                 patch('cargo_capture.subprocess.run') as compiler, \
                 patch('cargo_capture.subprocess.check_output', side_effect=['head\n', '']), \
                 self.assertRaises(CompilerUnitRefusal):
                runner(command, root, root/'refused.log')
            compiler.assert_not_called()
            text = (root/'refused.log').read_text()
            self.assertIn('End: ', text)
            self.assertIn('Exit code: 125', text)
            self.assertIn('drifted after declaration', text)

    def test_repeated_invocations_cannot_overwrite_retained_lane_evidence(self):
        first = retention_name('test-cost/store/build.log', '0123456789abcdef', '2026-01-01T00:00:01Z')
        second = retention_name('test-cost/store/build.log', '0123456789abcdef', '2026-01-01T00:00:02Z')
        self.assertNotEqual(first, second)
        self.assertLess(len(retention_name('x' * 1000, 'head', '2026-01-01T00:00:01Z')), 140)

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
