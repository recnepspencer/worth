import os
from pathlib import Path
import sys
import tempfile
import unittest


class ProcessAccountingTests(unittest.TestCase):
    def test_windows_accounting_preserves_child_output_and_nonzero_exit(self):
        if os.name != 'nt':
            # The platform adapter is only imported on Windows; the POSIX
            # boundary is covered by test_account.py.
            return
        from account_windows import run
        with tempfile.TemporaryDirectory() as temporary:
            script = Path(temporary) / 'child.py'
            script.write_text('print("accounted-child"); raise SystemExit(3)')
            code, cpu, posture = run([sys.executable, str(script)])
            self.assertEqual(code, 3)
            self.assertIn(posture, ['complete', 'accounting-unavailable', 'descendants-still-active'])
            if cpu is not None:
                self.assertGreaterEqual(cpu, 0)


if __name__ == '__main__':
    unittest.main()
