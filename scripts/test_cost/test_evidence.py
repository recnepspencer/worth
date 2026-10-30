import tempfile
from pathlib import Path
import sys
import unittest

from evidence import run


class LaneEvidenceTests(unittest.TestCase):
    def test_failed_child_retains_command_revision_output_and_exit(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'lane.log'
            command = [sys.executable, '-c', 'print("observed"); raise SystemExit(3)']
            self.assertEqual(run(command, path), 3)
            log = path.read_text()
            for field in ['Command:', 'HEAD:', 'Git status --short empty:',
                          'Start:', 'observed', 'End:', 'Exit code: 3']:
                self.assertIn(field, log)


if __name__ == '__main__':
    unittest.main()
