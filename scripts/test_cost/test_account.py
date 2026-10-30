import json
from pathlib import Path
import sys
import tempfile
import unittest

from account import execute


class ExactChildAccountingTests(unittest.TestCase):
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
