"""A historical layout diagnosis must never turn a failure into a retry."""
import io
from pathlib import Path
import sys
import unittest
from contextlib import redirect_stderr
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'test_cost'))
from account import execute


class LaunchDiagnosisTests(unittest.TestCase):
    def test_real_launch_error_is_diagnosed_once_and_propagated(self):
        error = ProcessLookupError('exec mapping refused')
        with patch('account.process', side_effect=error) as child, \
             patch('macos_link.diagnostic.diagnose', return_value='MACOS_EXEC_LAYOUT'), \
             patch('account.sys.platform', 'darwin'), redirect_stderr(io.StringIO()) as output:
            with self.assertRaises(ProcessLookupError) as caught:
                execute(['historical'], {})
        self.assertIs(caught.exception, error)
        child.assert_called_once_with(['historical'])
        self.assertIn('MACOS_EXEC_LAYOUT', output.getvalue())

    def test_unavailable_diagnosis_preserves_sigkill(self):
        with patch('account.process', return_value=(-9, 1, 'complete')) as child, \
             patch('account.sys.platform', 'darwin'), \
             patch('builtins.__import__', side_effect=ImportError('diagnostic unavailable')):
            code, _ = execute(['historical'], {})
        self.assertEqual(code, -9)
        child.assert_called_once()

    def test_sigkill_stays_sigkill_and_no_success_is_inspected(self):
        for code in (-9, 0):
            with self.subTest(code=code), patch('account.process', return_value=(code, 1, 'complete')) as child, \
                 patch('macos_link.diagnostic.diagnose', return_value='MACOS_EXEC_LAYOUT') as diagnosis, \
                 patch('account.sys.platform', 'darwin'), redirect_stderr(io.StringIO()):
                observed, _ = execute(['historical'], {})
                self.assertEqual(observed, code)
                child.assert_called_once()
                self.assertEqual(diagnosis.call_count, int(code == -9))
