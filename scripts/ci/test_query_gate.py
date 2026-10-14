"""Failure diagnostics exercise the real command owner without Cargo."""
import contextlib
import io
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

import query_gate


class FailureReporting(unittest.TestCase):
    def test_failed_command_retains_complete_log_and_reports_bounded_tail(self):
        child = (
            "import sys; "
            "print('EARLY'); "
            "print(('x' * 512 + '\\n') * 100); "
            "print('FINAL diagnostic', file=sys.stderr); sys.exit(7)"
        )
        with tempfile.TemporaryDirectory() as temporary:
            out = Path(temporary)
            console = io.StringIO()
            commands = {"failure-report": [("failed-child", [sys.executable, "-u", "-c", child])]}
            with patch.dict(query_gate.COMMAND_SUITES, commands), contextlib.redirect_stdout(console):
                result = query_gate.Gate(out).run("failure-report")
            self.assertFalse(result["ok"])
            self.assertEqual(result["commands"][0]["exit"], 7)
            self.assertEqual(json.loads((out / "failure-report.json").read_text()), result)
            complete = (out / "failed-child.log").read_text()
            self.assertIn("EARLY", complete)
            self.assertIn("FINAL diagnostic", complete)
            report = console.getvalue()
            header, tail = report.split("\n", 1)
            self.assertIn("failed-child: exit 7", header)
            self.assertIn(str((out / "failed-child.log").resolve()), header)
            self.assertNotIn("EARLY", tail)
            self.assertIn("FINAL diagnostic", tail)
            self.assertLessEqual(len(tail.encode("utf-8")), 16385)
            self.assertLessEqual(len(tail.splitlines()), 80)


if __name__ == "__main__":
    unittest.main()
