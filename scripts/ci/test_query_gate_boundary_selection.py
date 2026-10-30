"""Checking the constitution and testing its implementation are explicit lanes."""
import contextlib
import io
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import query_gate


class BoundaryCommandSelectionTests(unittest.TestCase):
    def test_boundary_validation_keeps_preparation_real_gate_and_caps_without_checker_tests(self):
        commands = query_gate.COMMAND_SUITES['boundary']
        self.assertEqual([name for name, _ in commands], ['prepare-constitution', 'constitution', 'line-caps'])
        self.assertTrue(commands[0][1][-1].endswith('scripts/prepare-constitution-hook.ps1'))
        self.assertTrue(commands[1][1][-1].endswith('scripts/check-constitution.ps1'))
        self.assertEqual(query_gate.COMMAND_ENV['constitution'], {'WORTH_CONSTITUTION_PREBUILT': '1'})
        self.assertFalse(any(command[:2] == ['cargo', 'test'] for _, command in commands))
        self.assertEqual(query_gate.COMMAND_SUITES['boundary-tests'], [
            ('boundary-tests', ['cargo', 'test', '--manifest-path', query_gate.BOUNDARY,
                                '--target-dir', 'target/constitution-tools'])])

    def test_real_gate_dispatch_preserves_separate_commands_and_failure_outcomes(self):
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary)
            for suite in ('boundary', 'boundary-tests'):
                with self.subTest(suite=suite):
                    gate = query_gate.Gate(output)
                    expected = query_gate.COMMAND_SUITES[suite]
                    seen = []
                    def command(label, argv):
                        seen.append((label, argv))
                        return {'name': label, 'exit': 7 if label == expected[-1][0] else 0,
                                'results': [], 'failed': [], 'seconds': 0, 'log': label + '.log'}
                    with patch.object(gate, 'command', side_effect=command):
                        result = gate.run(suite)
                    self.assertEqual(seen, expected)
                    self.assertFalse(result['ok'])
                    self.assertEqual(result['commands'][-1]['exit'], 7)

    def test_both_explicit_lanes_are_discoverable_without_constructing_a_gate(self):
        output = io.StringIO()
        with patch.object(query_gate, 'Gate') as gate, contextlib.redirect_stdout(output):
            self.assertEqual(query_gate.main(['--list']), 0)
        gate.assert_not_called()
        self.assertIn('boundary', output.getvalue().splitlines())
        self.assertIn('boundary-tests', output.getvalue().splitlines())

    def test_generic_full_order_retains_checker_tests_exactly_once(self):
        self.assertEqual(query_gate.ORDER.count('boundary-tests'), 1)
        self.assertLess(query_gate.ORDER.index('boundary'), query_gate.ORDER.index('boundary-tests'))

    def test_ci_full_boundary_job_keeps_validation_and_checker_self_tests(self):
        source = (query_gate.ROOT/'.github/workflows/ci.yml').read_text(encoding='utf-8')
        job = source.split('\n  boundary-check:\n', 1)[1].split('\n  query-checkpoint-recovery:\n', 1)[0]
        self.assertEqual(job.count('run: ./scripts/prepare-constitution-hook.ps1'), 1)
        self.assertEqual(job.count('run: ./scripts/check-constitution.ps1'), 1)
        self.assertEqual(job.count('run: cargo test --manifest-path tools/boundary-check/Cargo.toml '
                                   '--target-dir target/constitution-tools'), 1)


if __name__ == '__main__':
    unittest.main()
