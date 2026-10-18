"""Failure diagnostics exercise the real command owner without Cargo."""
import contextlib
import io
import json
from pathlib import Path
import sys
import tempfile
from types import SimpleNamespace
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


class DeclaredHarness(unittest.TestCase):
    def test_artifacts_use_target_kind_and_declared_harness_with_true_default(self):
        manifest_text = '''[package]
name = "real-package"
version = "0.1.0"
[lib]
name = "renamed_library"
harness = false
[[test]]
name = "shared"
harness = false
[[example]]
name = "shared"
harness = true
[[example]]
name = "main_example"
harness = false
[[bin]]
name = "shared"
harness = false
[[bench]]
name = "shared"
harness = false
[[test]]
name = "default_harness"
'''
        selections = [("lib", "renamed_library", False), ("test", "shared", False),
                      ("example", "shared", True), ("example", "main_example", False),
                      ("bin", "shared", False), ("bench", "shared", False),
                      ("test", "default_harness", True), ("bin", "automatic", True)]
        with tempfile.TemporaryDirectory() as temporary:
            manifest = Path(temporary) / "Cargo.toml"
            manifest.write_text(manifest_text, encoding="utf-8")
            records = [json.dumps({
                "reason": "compiler-artifact", "manifest_path": str(manifest),
                "target": {"kind": [kind], "name": name},
                "profile": {"test": True}, "executable": name,
            }) for kind, name, _ in selections]
            binaries = query_gate.test_binaries("\n".join(records))
        self.assertEqual([b["harness"] for b in binaries], [s[2] for s in selections])
        self.assertTrue(all(b["package"] == "real-package" for b in binaries))
        self.assertEqual([b["lib"] for b in binaries], [True] + [False] * 7)

    def test_harness_free_main_runs_once_without_listing_and_retains_failure(self):
        with tempfile.TemporaryDirectory() as temporary:
            out = Path(temporary)
            binary = {"package": "reference", "target": "main", "lib": False,
                      "harness": False, "exe": "reference-main", "cwd": temporary}
            gate = query_gate.Gate(out)
            gate.built["query"] = ([binary], 0, "existing-build.log")

            def run(argv, **kwargs):
                self.assertEqual(argv, ["reference-main"])
                self.assertEqual(kwargs["cwd"], temporary)
                kwargs["stdout"].write("real main failed once\n")
                return SimpleNamespace(returncode=7)

            with patch.object(query_gate.subprocess, "run", side_effect=run) as process:
                result = gate.run("query-rest")
            process.assert_called_once()
            self.assertFalse(result["ok"])
            self.assertEqual(result["chunks"][0]["exit"], 7)
            self.assertEqual(result["bad_exit"], ["query-rest-chunk-000.log"])
            self.assertEqual((out / result["bad_exit"][0]).read_text(), "real main failed once\n")
            self.assertEqual(json.loads((out / "query-rest.json").read_text()),
                             json.loads(json.dumps(result)))

    def test_empty_real_harness_listing_is_not_executed_as_a_main(self):
        with tempfile.TemporaryDirectory() as temporary:
            binary = {"package": "empty", "target": "tests", "lib": True,
                      "harness": True, "exe": "empty-tests", "cwd": temporary}
            gate = query_gate.Gate(Path(temporary))
            gate.built["query"] = ([binary], 0, "existing-build.log")
            with patch.object(query_gate.subprocess, "run",
                              return_value=SimpleNamespace(stdout="", returncode=0)) as process:
                result = gate.tests("query-rest")
            self.assertEqual(process.call_args.args[0],
                             ["empty-tests", "--list", "--format", "terse"])
            process.assert_called_once()
            self.assertEqual(result["chunks"], [])


if __name__ == "__main__":
    unittest.main()
