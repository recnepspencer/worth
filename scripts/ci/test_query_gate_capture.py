"""The gate delegates every Cargo process to the compiler capture owner."""
import contextlib
import io
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

import query_gate


class CargoCaptureHandoffTests(unittest.TestCase):
    def test_command_publishes_its_overrides_without_mutating_the_gate_environment(self):
        with tempfile.TemporaryDirectory() as temporary:
            gate = query_gate.Gate(Path(temporary))
            gate.env['RUSTC_WRAPPER'] = 'gate-wrapper'
            gate.env['WORTH_TEST_COST_CAPTURE_INDEX'] = 'capture-index.jsonl'
            argv = ['cargo', 'check', '--workspace']

            def run(command, cwd, log):
                self.assertEqual(command, argv)
                self.assertEqual(cwd, query_gate.ROOT)
                log.write_text('raw compiler JSON\n', encoding='utf-8')
                return SimpleNamespace(returncode=0)

            with patch.object(query_gate, 'CargoRunner') as owner:
                owner.return_value.side_effect = run
                result = gate.command('check-ui-workspace', argv)
            environment = owner.call_args.args[1]
            self.assertEqual(environment['RUSTC_WRAPPER'], '')
            self.assertEqual(environment['WORTH_TEST_COST_CAPTURE_INDEX'], 'capture-index.jsonl')
            self.assertEqual(gate.env['RUSTC_WRAPPER'], 'gate-wrapper')
            self.assertEqual(result['exit'], 0)
            owner.return_value.assert_called_once()

    def test_nonlanding_build_reads_raw_captured_stdout_and_builds_only_once(self):
        raw = '{"reason":"compiler-artifact","fresh":false}\n'
        binaries = [{'package': 'fixture', 'exe': 'independently-built.exe'}]
        with tempfile.TemporaryDirectory() as temporary:
            gate = query_gate.Gate(Path(temporary))

            def run(command, cwd, log):
                self.assertEqual(command[:2], ['cargo', 'test'])
                self.assertIn('--no-run', command)
                self.assertEqual(cwd, query_gate.ROOT)
                log.write_text('full capture header\n' + raw, encoding='utf-8')
                return SimpleNamespace(returncode=0, stdout=raw)

            with patch.object(query_gate, 'CargoRunner') as owner, \
                    patch.object(query_gate, 'test_binaries', return_value=binaries) as parse:
                owner.return_value.side_effect = run
                first = gate.build('fixture')
                second = gate.build('fixture')
            self.assertEqual(first, second)
            self.assertEqual(first[0], binaries)
            parse.assert_called_once_with(raw)
            owner.return_value.assert_called_once()
            self.assertIs(owner.call_args.args[1], gate.env)
            self.assertEqual((Path(temporary) / 'group-fixture-build.jsonl').read_text(), raw)

    def test_captured_command_failure_retains_the_log_and_failure_diagnostic(self):
        with tempfile.TemporaryDirectory() as temporary:
            gate = query_gate.Gate(Path(temporary))

            def run(command, cwd, log):
                log.write_text('full capture header\ncompiler refused contract\n', encoding='utf-8')
                return SimpleNamespace(returncode=7)

            console = io.StringIO()
            with patch.object(query_gate, 'CargoRunner') as owner, contextlib.redirect_stdout(console):
                owner.return_value.side_effect = run
                result = gate.command('clippy-query', ['cargo', 'clippy', '--all-targets'])
            self.assertEqual(result['exit'], 7)
            self.assertIn('compiler refused contract', console.getvalue())
            self.assertIn('full capture header', (Path(temporary) / result['log']).read_text())


if __name__ == '__main__':
    unittest.main()
