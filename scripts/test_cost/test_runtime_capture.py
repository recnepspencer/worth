"""Real child-process checks for interruption-safe nextest result capture."""
from pathlib import Path
import sys
import tempfile
import unittest

from runtime_capture import execute


class RuntimeCaptureTests(unittest.TestCase):
    def test_result_is_flushed_while_the_child_is_alive_and_is_not_duplicated(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            path = root / 'run.log'
            # The child cannot finish successfully until its first event is
            # visible in the parent's log. Buffered-at-exit capture fails this.
            script = (
                'from pathlib import Path\nimport sys,time\n'
                'print("completed-result", flush=True)\n'
                'deadline=time.monotonic()+5\n'
                'while "completed-result" not in Path(sys.argv[1]).read_text():\n'
                '    if time.monotonic() > deadline: sys.exit(2)\n'
                '    time.sleep(0.01)\n'
                'print("runtime-failure", file=sys.stderr, flush=True)\n'
                'sys.exit(7)\n')
            with path.open('w', encoding='utf-8') as log:
                result = execute([sys.executable, '-c', script, str(path)], root, None, log)
            self.assertEqual(result.returncode, 7)
            self.assertEqual(result.stderr, '')
            self.assertEqual(path.read_text(), result.stdout)
            self.assertEqual(result.stdout.count('completed-result'), 1)
            self.assertIn('runtime-failure', result.stdout)


if __name__ == '__main__':
    unittest.main()
