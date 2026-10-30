import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch
from contextlib import redirect_stderr
import io

from account import execute


class ExactChildAccountingTests(unittest.TestCase):
    def test_unusable_reservations_preserve_the_exact_child_outcome_and_single_execution(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            invalid = root / 'invalid.json'
            invalid.write_text('{broken')
            for path in (invalid, root / 'absent.json'):
                with self.subTest(path=path), patch('account.process', return_value=(17, 2, 'complete')) as child:
                    with redirect_stderr(io.StringIO()) as diagnostic:
                        code, event = execute(['child'], {'WORTH_TEST_COST_RESERVATIONS': str(path)})
                    self.assertEqual(code, 17)
                    self.assertEqual(child.call_count, 1)
                    self.assertEqual(event['cpu_seconds'], 2)
                    self.assertEqual(event['accounting'], 'observation-unavailable')
                    self.assertEqual(event['observation_errors'][0]['kind'], 'reservation-unavailable')
                    self.assertIn('test-cost-observation-error', diagnostic.getvalue())

    def test_unwritable_accounting_destination_cannot_fail_a_successful_child(self):
        with tempfile.TemporaryDirectory() as temporary:
            destination = Path(temporary) / 'file'
            destination.write_text('an ordinary file cannot become a directory')
            with patch('account.process', return_value=(0, 1, 'complete')) as child, redirect_stderr(io.StringIO()):
                code, event = execute(['child'], {'WORTH_TEST_COST_ACCOUNTING_DIRECTORY': str(destination)})
            self.assertEqual(code, 0)
            self.assertEqual(child.call_count, 1)
            self.assertEqual(event['observation_errors'][0]['kind'], 'accounting-output-unavailable')
            self.assertEqual(destination.read_text(), 'an ordinary file cannot become a directory')

    def test_real_child_launch_failures_are_not_optional_telemetry_failures(self):
        with patch('account.process', side_effect=FileNotFoundError('child is absent')):
            with self.assertRaises(FileNotFoundError):
                execute(['absent-child'], {})

    def test_wrapper_runs_child_once_and_retains_cpu_as_telemetry(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            count = root / 'invocations.txt'
            script = root / 'child.py'
            script.write_text('from pathlib import Path\nimport sys\np=Path(sys.argv[1])\n'
                              'p.write_text(p.read_text()+"once" if p.exists() else "once")\n'
                              'raise SystemExit(3)\n')
            slots = root / 'slots.json'
            slots.write_text(json.dumps({'court\nbehaviour': 'num-test-threads'}))
            environment = {'NEXTEST_BINARY_ID': 'court', 'NEXTEST_TEST_NAME': 'behaviour',
                           'NEXTEST_TEST_THREADS': '8', 'WORTH_TEST_COST_RESERVATIONS': str(slots),
                           'WORTH_TEST_COST_ACCOUNTING_DIRECTORY': str(root / 'accounting')}
            code, event = execute([sys.executable, str(script), str(count)], environment)
            self.assertEqual(code, 3)
            self.assertEqual(count.read_text(), 'once')
            self.assertEqual(event['reserved_slots'], 8)
            records = list((root / 'accounting').glob('*.json'))
            self.assertEqual(len(records), 1)
            self.assertEqual(json.loads(records[0].read_text())['exit'], 3)


if __name__ == '__main__':
    unittest.main()
