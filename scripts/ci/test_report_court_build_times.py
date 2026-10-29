"""Own compiler-unit evidence, using Cargo 1.94's serialized UnitData schema."""

from contextlib import redirect_stderr, redirect_stdout
import io
import json
from pathlib import Path
import tempfile
import unittest

import report_court_build_times as report


COURT = "worth-query-court-host-fast"
TARGET = ' test "worth_query_court_host_fast" (test)'


def unit(index=0, name=COURT, target=TARGET, seconds=121, start=800):
    # Source schema: Cargo commit 85eff7c, timings::UnitData. `mode` really
    # is "todo"; trusting it as a Rust compilation mode would miss this test.
    return {"i": index, "name": name, "version": "0.1.0", "mode": "todo",
            "target": target, "features": [], "start": start, "duration": seconds,
            "unblocked_units": [], "unblocked_rmeta_units": [], "sections": None}


def html(units):
    return "<html><script>const UNIT_DATA = " + json.dumps(units) + ";</script></html>"


class CourtTimingTests(unittest.TestCase):
    def setUp(self):
        self.metadata = {COURT: {"boundary": "host", "class": "fast"},
                         "worth-query-worlds": {"boundary": None, "class": None}}

    def summarize(self, units, cold=True):
        return report.summarize(report.parse_units(html(units)), self.metadata,
                                {COURT: 4}, {COURT: {"publication": 4}}, cold)

    def test_real_schema_excludes_dependency_wait_and_other_target_work(self):
        units = [unit(), unit(1, "worth-query-execution", "", 900),
                 unit(2, target=' test "exception" (test)', seconds=20)]
        row = self.summarize(units)[0]
        self.assertEqual(row["integration_seconds"], 121)
        self.assertEqual(row["own_total_seconds"], 141)
        self.assertEqual(row["decision"], "split_candidate")
        self.assertEqual(len(row["targets"]), 2)
        self.assertEqual(row["planned_owners"], {"publication": 4})

    def test_split_threshold_is_strict_and_needs_clean_claim(self):
        self.assertEqual(self.summarize([unit(seconds=120)])[0]["decision"], "within_budget")
        self.assertEqual(self.summarize([unit()], False)[0]["decision"], "unqualified_cache")
        # Small count and short duration never decide an upward merge.
        self.assertEqual(self.summarize([unit(seconds=2)])[0]["decision"], "within_budget")

    def test_worlds_library_and_missing_canonical_test_are_distinct(self):
        rows = self.summarize([unit(target=' test "exception" (test)'),
                               unit(1, "worth-query-worlds", "", 125)])
        self.assertEqual(rows[0]["decision"], "unmeasured_canonical_target")
        self.assertEqual(rows[1]["library_seconds"], 125)
        self.assertEqual(rows[1]["decision"], "split_candidate")

    def test_name_classes_include_workspace_and_compile_boundary(self):
        self.assertEqual(report.court_identity("worth-query-bank-court-server-scale"),
                         ("worth-query-bank", "server", "scale"))
        self.assertEqual(report.court_identity("worth-query-court-compile-fail"),
                         ("worth-query", None, "compile_fail"))
        self.assertIsNone(report.court_identity("worth-query-host"))
        with self.assertRaisesRegex(report.ReportError, "invalid court"):
            report.court_identity("worth-query-court-host-serial")

    def test_bad_html_schema_and_duplicate_units_fail_explicitly(self):
        bad = ["no data", "const UNIT_DATA = [];", "<script>const UNIT_DATA = nope;</script>",
               "<script>const UNIT_DATA = [];</script>", html([unit(), unit()]),
               html([unit(seconds=-1)]), html([unit(seconds=float("nan"))]),
               html([{"i": 0}]), html([unit()]) + html([unit(1)])]
        for document in bad:
            with self.subTest(document=document[:80]), self.assertRaises(report.ReportError):
                report.parse_units(document)
        with self.assertRaisesRegex(report.ReportError, "duplicate target"):
            self.summarize([unit(), unit(1)])

    def test_cli_keeps_independent_runs_and_loads_metadata_counts(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            manifest = root / "crates" / COURT / "Cargo.toml"
            manifest.parent.mkdir(parents=True)
            manifest.write_text('[package]\nname = "' + COURT + '"\n'
                                '[package.metadata.worth.court]\nboundary = "host"\nclass = "fast"\n',
                                encoding="utf-8")
            plan = root / "plan.json"
            plan.write_text(json.dumps([{"court": COURT, "court_after": "fast::publication::case",
                                         "owner": "publication"}]), encoding="utf-8")
            inputs = [root / "cold.html", root / "another.html"]
            for index, path in enumerate(inputs):
                path.write_text(html([unit(seconds=122 + index)]), encoding="utf-8")
            output = io.StringIO()
            args = [*(str(path) for path in inputs), "--root", str(root), "--plan", str(plan),
                    "--cold", "--json"]
            with redirect_stdout(output):
                self.assertEqual(report.main(args), 0)
            result = json.loads(output.getvalue())
            self.assertEqual([r["packages"][0]["integration_seconds"] for r in result["runs"]],
                             [122, 123])
            self.assertEqual(result["runs"][0]["packages"][0]["tests"], 1)
            table = io.StringIO()
            with redirect_stdout(table):
                report.render_table(result)
            self.assertIn("Own total", table.getvalue())
            self.assertIn('test "worth_query_court_host_fast" (test): 122.00s', table.getvalue())
            with redirect_stderr(io.StringIO()):
                self.assertEqual(report.main([str(inputs[0]), str(inputs[0]), "--root", str(root)]), 2)
                self.assertEqual(report.main([str(root / "missing.html"), "--root", str(root)]), 2)
            # Duplicate occurrences must not inflate the planning-time count.
            plan.write_text(json.dumps([{"court": COURT, "court_after": "fast::case"}] * 2),
                            encoding="utf-8")
            with self.assertRaisesRegex(report.ReportError, "duplicate plan"):
                report.read_plan(plan)


if __name__ == "__main__":
    unittest.main()
