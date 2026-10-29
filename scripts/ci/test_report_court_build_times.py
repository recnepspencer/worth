"""Own compiler-unit evidence, using Cargo 1.94's serialized UnitData schema."""

from contextlib import redirect_stderr, redirect_stdout
import io
import json
from pathlib import Path
import tempfile
import unittest

import report_court_build_times as report


COURT = "worth-query-court-host"
TARGET = ' test "worth_query_court_host" (test)'


def unit(index=0, name=COURT, target=TARGET, seconds=121, start=800):
    # Source schema: Cargo commit 85eff7c, timings::UnitData. `mode` really
    # is "todo"; trusting it as a Rust compilation mode would miss this test.
    return {"i": index, "name": name, "version": "0.1.0", "mode": "todo",
            "target": target, "features": [], "start": start, "duration": seconds,
            "unblocked_units": [], "unblocked_rmeta_units": [], "sections": None}


def html(units):
    return "<html><script>const UNIT_DATA = " + json.dumps(units) + ";</script></html>"


class CourtTimingTests(unittest.TestCase):
    def test_testkit_family_records_library_telemetry_without_a_decision(self):
        name = "worth-query-testkit-host"
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            manifest = root / "crates" / name / "Cargo.toml"
            manifest.parent.mkdir(parents=True)
            manifest.write_text(f'[package]\nname = "{name}"\n',
                                encoding="utf-8")
            metadata = report.read_metadata(root)
        rows = report.summarize([unit(name=name, target="", seconds=121)],
                                metadata, {}, {})
        self.assertEqual(rows[0]["kind"], "testkit")
        self.assertEqual(rows[0]["library_seconds"], 121)
        self.assertNotIn("decision", rows[0])

    def test_metadata_reads_only_configured_workspaces(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            config = root / "tools/boundary-check/config/test-workspaces.txt"
            config.parent.mkdir(parents=True)
            config.write_text("workspaces/query | worth-query\n", encoding="utf-8")
            selected = root / "workspaces/query/crates/worth-query-testkit-host/Cargo.toml"
            selected.parent.mkdir(parents=True)
            selected.write_text('[package]\nname="worth-query-testkit-host"\n', encoding="utf-8")
            for relative in ("crates/outside-court-host/Cargo.toml",
                             "workspaces/another/crates/outside-testkit/Cargo.toml"):
                outside = root / relative
                outside.parent.mkdir(parents=True)
                outside.write_text("deliberately invalid TOML", encoding="utf-8")
            self.assertEqual(report.read_metadata(root),
                             {"worth-query-testkit-host": {"surface": None}})

    def setUp(self):
        self.metadata = {COURT: {"surface": "host"},
                         "worth-query-testkit": {"surface": None}}

    def summarize(self, units):
        return report.summarize(report.parse_units(html(units)), self.metadata,
                                {COURT: 4}, {COURT: {"publication": 4}})

    def test_real_schema_excludes_dependency_wait_and_other_target_work(self):
        units = [unit(), unit(1, "worth-query-execution", "", 900),
                 unit(2, target=' test "exception" (test)', seconds=20)]
        row = self.summarize(units)[0]
        self.assertEqual(row["integration_seconds"], 121)
        self.assertEqual(row["own_total_seconds"], 141)
        self.assertNotIn("decision", row)
        self.assertEqual(len(row["targets"]), 2)
        self.assertEqual(row["planned_owners"], {"publication": 4})

    def test_duration_never_changes_topology_or_reports_a_threshold(self):
        for seconds in (2, 120, 121, 10000):
            row = self.summarize([unit(seconds=seconds)])[0]
            self.assertEqual(row["integration_seconds"], seconds)
            self.assertNotIn("decision", row)
            self.assertFalse(any("budget" in key for key in row))

    def test_worlds_library_and_missing_canonical_test_are_distinct(self):
        rows = self.summarize([unit(target=' test "exception" (test)'),
                               unit(1, "worth-query-testkit", "", 125)])
        self.assertIsNone(rows[0]["integration_seconds"])
        self.assertEqual(rows[1]["library_seconds"], 125)
        self.assertNotIn("decision", rows[1])

    def test_names_describe_surfaces_without_classes(self):
        self.assertEqual(report.court_identity("worth-query-bank-court-server"),
                         ("worth-query-bank", "server"))
        self.assertEqual(report.court_identity("worth-query-court-archive-wire"),
                         ("worth-query", "archive-wire"))
        self.assertIsNone(report.court_identity("worth-query-host"))
        with self.assertRaisesRegex(report.ReportError, "invalid court"):
            report.court_identity("worth-query-court-")

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
                                '[package.metadata.worth.court]\nsurface = "host"\n',
                                encoding="utf-8")
            plan = root / "plan.json"
            plan.write_text(json.dumps([{"court": COURT, "court_after": "fast::publication::case",
                                         "owner": "publication"}]), encoding="utf-8")
            inputs = [root / "cold.html", root / "another.html"]
            for index, path in enumerate(inputs):
                path.write_text(html([unit(seconds=122 + index)]), encoding="utf-8")
            output = io.StringIO()
            args = [*(str(path) for path in inputs), "--root", str(root), "--plan", str(plan),
                    "--cache-conditions", "warm dependencies, new court targets", "--json"]
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
            self.assertIn('test "worth_query_court_host" (test): 122.00s', table.getvalue())
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
