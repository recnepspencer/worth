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
import query_gate_courts


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
    def test_fixture_build_includes_existing_integration_contracts(self):
        self.assertIn("--tests", query_gate_courts.GROUPS["fixture"])
        self.assertIn("worth-query-topology-entry", query_gate_courts.GROUPS["fixture"])

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


class CostClassScheduling(unittest.TestCase):
    def test_failed_listing_cannot_silently_drop_a_binary(self):
        with tempfile.TemporaryDirectory() as temporary:
            binary = {"package": "example", "target": "example", "lib": False,
                      "harness": True, "exe": "broken-tests", "cwd": temporary}
            gate = query_gate.Gate(Path(temporary))
            gate.built["query"] = ([binary], 0, "existing-build.log")
            with patch.object(query_gate.subprocess, "run", return_value=SimpleNamespace(
                    stdout="", stderr="loader refused", returncode=7)):
                result = gate.tests("query-rest")
            self.assertFalse(result["ok"])
            self.assertEqual(result["listing"], "FAILED")
            self.assertIn("loader refused", (Path(temporary) / result["log"]).read_text())

    def test_renamed_classed_cases_keep_isolation_and_complete_coverage(self):
        names = ["fast::new_name", "serial::renamed_counter",
                 "owner::serial::renamed_unit", "compile_fail::renamed_contract"]
        with tempfile.TemporaryDirectory() as temporary:
            binary = {"package": "example", "target": "example", "lib": False,
                      "harness": True, "exe": "example-tests", "cwd": temporary}
            gate = query_gate.Gate(Path(temporary))
            gate.built["query"] = ([binary], 0, "existing-build.log")
            chunks = []

            def chunk(suite, index, selected, cases, extra=()):
                chunks.append((cases, extra))
                return {"failed": [], "exit": 0, "passed": len(cases),
                        "seconds": 0, "log": f"chunk-{index}.log"}

            listed = "\n".join(name + ": test" for name in names)
            with patch.object(query_gate.subprocess, "run",
                              return_value=SimpleNamespace(stdout=listed, returncode=0)), \
                    patch.object(gate, "chunk", side_effect=chunk):
                result = gate.tests("query-rest")
            self.assertTrue(result["ok"])
            self.assertCountEqual([n for cases, _ in chunks for n in cases], names)
            self.assertTrue(all(len(cases) == 1 for cases, _ in chunks))
            self.assertEqual(chunks[0][0], ["fast::new_name"])

    def test_exhaustive_fixture_paths_remain_cpu_exclusive_after_rename(self):
        names = ["serial::renamed_fixture", "owner::exhaustive::renamed_matrix"]
        with tempfile.TemporaryDirectory() as temporary:
            binary = {"package": "worth-query-topology-entry", "target": "topology",
                      "lib": True, "harness": True, "exe": "fixture", "cwd": temporary}
            gate = query_gate.Gate(Path(temporary))
            gate.built["fixture"] = ([binary], 0, "existing-build.log")
            executed = []

            def chunk(suite, index, selected, cases, extra=()):
                executed.extend(cases)
                return {"failed": [], "exit": 0, "passed": len(cases),
                        "seconds": 0, "log": f"chunk-{index}.log"}

            listed = "\n".join(name + ": test" for name in names)
            with patch.object(query_gate.subprocess, "run",
                              return_value=SimpleNamespace(stdout=listed, returncode=0)), \
                    patch.object(gate, "chunk", side_effect=chunk):
                result = gate.tests("fixture")
            self.assertTrue(result["ok"])
            self.assertEqual(executed, names)


class CourtSelection(unittest.TestCase):
    def test_category_suites_do_not_repeat_harness_free_product_examples(self):
        with tempfile.TemporaryDirectory() as temporary:
            binary = {"package": "owning-example", "target": "main", "lib": False,
                      "harness": False, "exe": "reference-main", "cwd": temporary}
            gate = query_gate.Gate(Path(temporary))
            gate.built["query"] = ([binary], 0, "existing-build.log")
            with patch.object(query_gate.subprocess, "run") as process:
                result = gate.tests("query-process")
            process.assert_not_called()
            self.assertTrue(result["ok"])
            self.assertEqual(result["chunks"], [])

    def test_explicit_build_identity_reaches_the_complete_gate(self):
        identity = "00000000-0000-4000-8000-000000000001"
        with tempfile.TemporaryDirectory() as temporary, patch.dict(
                query_gate.os.environ, {"WORTH_IMPLEMENTATION_BUILD_ID": identity}):
            gate = query_gate.Gate(Path(temporary))
        self.assertEqual(gate.env["WORTH_IMPLEMENTATION_BUILD_ID"], identity)

    def test_build_selection_is_shared_by_every_category(self):
        selection = query_gate_courts.ordinary_selection(query_gate.QUERY)
        self.assertIn("--workspace", selection)
        self.assertIn("--tests", selection)
        self.assertNotIn("--exclude", selection)
        self.assertNotIn("--lib", selection)
        for suite in ("query-process", "query-exhaustive", "query-determinism"):
            self.assertEqual(query_gate.TEST_SUITES[suite][0], "query")

    def test_category_suites_partition_paths_without_classifying_behaviour_names(self):
        binary = {"package": "worth-query-court-host"}
        cases = ["ordinary::process_word_is_a_behaviour", "process::io::behaviour",
                 "oracle::reference::behaviour", "cost::growth::behaviour",
                 "determinism::replay::behaviour", "compile_fail::sealed::behaviour"]
        suites = ["query-rest", "query-process", "query-exhaustive", "query-scale",
                  "query-determinism", "query-compile-fail"]
        for name, expected in zip(cases, suites):
            selected = [suite for suite in suites
                        if query_gate_courts.selected(suite, binary, name)]
            self.assertEqual(selected, [expected])



if __name__ == "__main__":
    unittest.main()
