import copy
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import types
import unittest
from unittest import mock


SCRIPT = Path(__file__).with_name("compare_signal_performance.py")
if str(SCRIPT.parent) not in sys.path:
    sys.path.insert(0, str(SCRIPT.parent))
SPEC = importlib.util.spec_from_file_location("signal_perf_compare", SCRIPT)
PERF = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(PERF)
from signal_performance_test_fixtures import (
    capture, matched_captures, set_chain_metric, set_metric,
)


class ComparisonTests(unittest.TestCase):
    def test_frozen_resolution_and_f3_phase_roster_are_exact(self):
        self.assertEqual(PERF.CASE_RESOLUTION, {
            "strict_heavy": (21, 4), "median_only": (21, 4), "structural_only": (3, 0),
        })
        self.assertEqual(PERF.TESTS, [
            PERF.FAMILY + name for name in (
                "chain_bootstrap::perf_chain_10k_bootstrap_serial",
                "dependency_reconciliation_rotating::perf_dependency_reconciliation_rotating_window_serial",
                "dependency_reconciliation_stable_shape::perf_dependency_reconciliation_stable_shape_staged_serial",
                "dependency_reconciliation_staged::perf_dependency_reconciliation_rotating_window_staged_serial",
                "fintech_fanout::perf_fintech_mixed_fanout_profile_matrix",
                "observability_profile::perf_harness_observability_profile_delta",
                "suppression_fanout::perf_suppression_wide_fanout_serial",
                "topology_rewiring::perf_topology_rewiring_churn_serial",
                "topology_rewiring::perf_topology_rewiring_rotating_window_serial",
            )
        ])
        case_resolutions = [
            (suite, profile, *PERF.CONTRACTS[(suite, profile)][:3])
            for suite, profile in PERF.CASES
        ]
        self.assertEqual(case_resolutions, [
            ("chain_10k_bootstrap", "balanced", "median_only", 21, 4),
            ("dependency_reconciliation_rotating_window", "balanced", "structural_only", 3, 0),
            ("dependency_reconciliation_stable_shape_staged", "balanced", "structural_only", 3, 0),
            ("dependency_reconciliation_rotating_window_staged", "balanced", "structural_only", 3, 0),
            ("fintech_mixed_fanout", "operational", "strict_heavy", 21, 4),
            ("fintech_mixed_fanout", "development", "median_only", 21, 4),
            ("fintech_mixed_fanout", "forensic", "median_only", 21, 4),
            ("harness_observability_profile", "development", "structural_only", 3, 0),
            ("harness_observability_profile", "forensic", "structural_only", 3, 0),
            ("suppression_wide_fanout", "balanced", "median_only", 21, 4),
            ("topology_rewiring_churn", "balanced", "median_only", 21, 4),
            ("topology_rewiring_rotating_window", "balanced", "median_only", 21, 4),
        ])
        self.assertEqual(PERF.CONTRACTS[("fintech_mixed_fanout", "operational")][1:3],
                         (21, 4))
        self.assertEqual(PERF.CONTRACTS[("fintech_mixed_fanout", "development")][1:3],
                         (21, 4))
        self.assertEqual(
            PERF.CONTRACTS[("dependency_reconciliation_rotating_window", "balanced")][1:3],
            (3, 0),
        )
        chain_phases = PERF.CONTRACTS[("chain_10k_bootstrap", "balanced")][3]
        self.assertIn("push_nanos", chain_phases)
        chain = capture()["cases"]["ordinary"][0]
        self.assertEqual(
            chain["contract"]["scoped_allocation_metrics"],
            ["push_scoped_allocation_calls", "push_scoped_requested_bytes"],
        )
        self.assertEqual(
            chain["relative_budgets"]["metrics.push_scoped_requested_bytes"],
            {"median": 1.10, "max": 1.10},
        )
        self.assertEqual(PERF.CAPTURE_ORDER, ("A1", "B1", "B2", "A2"))

    def test_two_capture_comparison_is_diagnostic_never_performance_acceptance(self):
        report, code = PERF.compare(capture(), capture())
        self.assertEqual(code, 0)
        self.assertEqual(report["status"], "diagnostic_within_budgets")
        self.assertFalse(report["performance_acceptance"])
        self.assertIn("requires_full_matched_aa", report["acceptance_posture"])
        self.assertEqual(report["relative_verdict"], "within budgets")
        self.assertEqual(report["absolute_verdict"], "within contracts")

    def test_relative_boundary_and_regression_use_unchanged_allocation_budget(self):
        baseline = capture(1_000)
        at_boundary = capture(1_100)
        report, code = PERF.compare(baseline, at_boundary)
        self.assertEqual(code, 0, report)

        over_budget = capture(1_101)
        report, code = PERF.compare(baseline, over_budget)
        self.assertEqual(code, 1)
        self.assertTrue(any(item["metric"].endswith("allocated_bytes")
                            for item in report["relative_regressions"]))

    def test_absolute_baseline_debt_is_separate_from_relative_verdict(self):
        baseline = capture(access_value=1)
        report, code = PERF.compare(baseline, copy.deepcopy(baseline))
        self.assertEqual(code, 4)
        self.assertEqual(report["relative_verdict"], "within budgets")
        self.assertEqual(report["absolute_verdict"], "violation")
        self.assertEqual(report["status"], "rejected")

    def test_missing_case_metric_and_wrong_budget_are_invalid(self):
        missing_case = capture()
        missing_case["cases"]["ordinary"].pop()
        with self.assertRaisesRegex(ValueError, "twelve-case"):
            PERF.validate_capture(missing_case)

        missing_metric = capture()
        del missing_metric["cases"]["ordinary"][0]["samples"][0]["metrics"]["allocation_metrics"]["allocation_calls"]
        with self.assertRaisesRegex(ValueError, "metric roster"):
            PERF.validate_capture(missing_metric)

        wrong_budget = capture()
        wrong_budget["cases"]["ordinary"][0]["relative_budgets"]["elapsed_micros"]["median"] = 99
        with self.assertRaisesRegex(ValueError, "budget authority"):
            PERF.validate_capture(wrong_budget)

    def test_wrong_environment_and_corrupt_json_are_invalid(self):
        baseline, candidate = capture(), capture()
        candidate["environment"]["host"] = "different"
        with self.assertRaisesRegex(ValueError, "environment mismatch"):
            PERF.compare(baseline, candidate)
        with self.assertRaisesRegex(ValueError, "duplicate JSON key"):
            PERF.decode('{"status":"captured","status":"forged"}')
        with self.assertRaisesRegex(ValueError, "invalid number"):
            PERF.decode('{"nan":NaN}')
        with self.assertRaisesRegex(ValueError, "invalid number"):
            PERF.decode('{"overflow":1e400}')

    def test_extra_command_flags_zero_peak_and_warmup_drift_are_invalid(self):
        wrong_command = capture()
        wrong_command["commands"]["ordinary"]["argv"][2:2] = ["--config", "profile.release.opt-level=0"]
        with self.assertRaisesRegex(ValueError, "argv length"):
            PERF.validate_capture(wrong_command)

        zero_peak = capture()
        for row in zero_peak["cases"]["peak"][0]["samples"]:
            row["metrics"]["allocation_metrics"]["peak_live_bytes"] = 0
        with self.assertRaisesRegex(ValueError, "tracker recorded zero"):
            PERF.validate_capture(zero_peak)

        warmup_drift = capture()
        warmup_drift["cases"]["ordinary"][0]["workload_warmup"] = "none"
        with self.assertRaisesRegex(ValueError, "Once warmup posture"):
            PERF.validate_capture(warmup_drift)

        malformed_environment = capture()
        malformed_environment["environment"]["logical_cpus"] = None
        with self.assertRaisesRegex(ValueError, "logical CPU"):
            PERF.validate_capture(malformed_environment)
        missing_cpu = capture()
        missing_cpu["environment"]["cpu"] = "unknown"
        with self.assertRaisesRegex(ValueError, "environment field: cpu"):
            PERF.validate_capture(missing_cpu)

        with mock.patch.dict(os.environ, {"WORTH_SIGNAL_PERF_SAMPLES": "99"}):
            with self.assertRaisesRegex(ValueError, "ambient WORTH_SIGNAL"):
                PERF.environment(SCRIPT.parents[2])

    def test_peak_probe_cannot_omit_or_invent_metric_roster(self):
        missing = capture()
        for row in missing["cases"]["peak"][0]["samples"]:
            del row["metrics"]["access_counters"]
        with self.assertRaisesRegex(ValueError, "sample metric roster differs"):
            PERF.validate_capture(missing)

        invented = capture()
        for row in invented["cases"]["peak"][0]["samples"]:
            row["metrics"]["uncontracted_peak_only_metric"] = 1
        with self.assertRaisesRegex(ValueError, "sample metric roster differs"):
            PERF.validate_capture(invented)

    def test_frozen_integer_schema_and_probe_paths_reject_python_aliases(self):
        mutations = (
            ("version", lambda data: data.__setitem__("version", 2.0)),
            ("configuration", lambda data: data["configuration"].__setitem__("test_threads", True)),
            ("sequence", lambda data: data["matched_capture"].__setitem__("sequence_index", False)),
            ("returncode", lambda data: data["commands"]["ordinary"].__setitem__("returncode", False)),
            ("sample_count", lambda data: data["cases"]["ordinary"][1].__setitem__("sample_count", 3.0)),
            ("warmup_count", lambda data: data["cases"]["ordinary"][1].__setitem__("warmup_count", False)),
            ("protocol_count", lambda data: data["cases"]["ordinary"][1]["measurement_protocol"].__setitem__("sample_count", 3.0)),
            ("maximum", lambda data: data["cases"]["ordinary"][0]["contract"]["access_counter_maxima"][0].__setitem__(1, 0.0)),
            ("peak_target", lambda data: data["commands"]["peak"]["argv"].__setitem__(7, "C:/other/target")),
        )
        for name, mutate in mutations:
            with self.subTest(name=name), self.assertRaises(ValueError):
                evidence = copy.deepcopy(capture())
                mutate(evidence)
                PERF.validate_capture(evidence)

    def test_sample_metrics_outside_rust_u128_are_invalid(self):
        evidence = capture()
        evidence["cases"]["ordinary"][0]["samples"][0]["elapsed_micros"] = 10 ** 1000
        with self.assertRaisesRegex(ValueError, "u128"):
            PERF.validate_capture(evidence)

    def test_capture_preserves_build_failure_as_noncomparison_evidence(self):
        with tempfile.TemporaryDirectory() as temporary:
            temporary = Path(temporary)
            args = types.SimpleNamespace(
                root=SCRIPT.parents[2],
                target_dir=temporary / "target",
                output=temporary / "capture.json",
                matched_set="failed-set",
                slot="A1",
            )
            failed = subprocess.CompletedProcess([], 23, stdout="compiler failed\n")
            with mock.patch.object(PERF, "environment", return_value={"host": "test"}), \
                    mock.patch.object(PERF.subprocess, "run", return_value=failed):
                self.assertEqual(PERF.capture(args), 3)
            evidence = json.loads(args.output.read_text())
            self.assertEqual(evidence["status"], "benchmark_failed")
            self.assertIn("build/list failed", evidence["error"])
            self.assertNotIn("cases", evidence["commands"])

    def test_matched_equal_captures_resolve_without_hiding_any_slot(self):
        report, code = PERF.compare_matched(
            capture(slot="A1"), capture(slot="B1"),
            capture(slot="B2"), capture(slot="A2"),
        )
        self.assertEqual(code, 0, report)
        self.assertEqual(report["capture_slots"], ["A1", "B1", "B2", "A2"])
        self.assertEqual(report["relative_verdict"], "resolved_within_relative_budgets")
        self.assertTrue(report["performance_acceptance"])

    def test_matched_f2_claim_is_inconclusive_when_aa_noise_exceeds_headroom(self):
        captures = matched_captures()
        set_metric(captures["A1"], "mutation_nanos", 857_800)
        set_metric(captures["A2"], "mutation_nanos", 807_000)
        set_metric(captures["B1"], "mutation_nanos", 979_200)
        set_metric(captures["B2"], "mutation_nanos", 979_200)
        report, code = PERF.compare_matched(
            captures["A1"], captures["B1"], captures["B2"], captures["A2"]
        )
        self.assertEqual(code, 5, report)
        claim = next(item for item in report["inconclusive_noise"]
                     if item["metric"] == "metrics.mutation_nanos")
        self.assertGreater(claim["baseline_noise"], claim["remaining_headroom"])
        self.assertEqual(report["status"], "inconclusive")
        self.assertFalse(report["performance_acceptance"])

    def test_matched_f3_historical_push_cost_is_an_explicit_regression(self):
        captures = matched_captures()
        set_chain_metric(captures["A1"], "push_nanos", 339_800)
        set_chain_metric(captures["A2"], "push_nanos", 378_300)
        set_chain_metric(captures["B1"], "push_nanos", 572_600)
        set_chain_metric(captures["B2"], "push_nanos", 572_600)
        report, code = PERF.compare_matched(
            captures["A1"], captures["B1"], captures["B2"], captures["A2"]
        )
        self.assertEqual(code, 1, report)
        self.assertTrue(any(
            item["metric"] == "metrics.push_nanos"
            for item in report["relative_regressions"]
        ))

    def test_matched_f3_scoped_allocation_uses_unchanged_allocation_budget(self):
        captures = matched_captures()
        for slot in ("A1", "A2"):
            set_chain_metric(captures[slot], "push_scoped_requested_bytes", 100)
        for slot in ("B1", "B2"):
            set_chain_metric(captures[slot], "push_scoped_requested_bytes", 111)
        report, code = PERF.compare_matched(
            captures["A1"], captures["B1"], captures["B2"], captures["A2"]
        )
        self.assertEqual(code, 1, report)
        self.assertTrue(any(item["metric"] == "metrics.push_scoped_requested_bytes"
                            for item in report["relative_regressions"]))

    def test_matched_above_budget_is_regression_even_when_noise_is_large(self):
        captures = matched_captures()
        set_metric(captures["A1"], "mutation_nanos", 100)
        set_metric(captures["A2"], "mutation_nanos", 106)
        set_metric(captures["B1"], "mutation_nanos", 126)
        set_metric(captures["B2"], "mutation_nanos", 121)
        report, code = PERF.compare_matched(
            captures["A1"], captures["B1"], captures["B2"], captures["A2"]
        )
        self.assertEqual(code, 1, report)
        self.assertTrue(any(item["metric"] == "metrics.mutation_nanos"
                            for item in report["relative_regressions"]))

    def test_matched_noise_equal_to_headroom_is_resolved_exactly(self):
        captures = matched_captures()
        set_metric(captures["A1"], "mutation_nanos", 100)
        set_metric(captures["A2"], "mutation_nanos", 104)
        set_metric(captures["B1"], "mutation_nanos", 120)
        set_metric(captures["B2"], "mutation_nanos", 124)
        report, code = PERF.compare_matched(
            captures["A1"], captures["B1"], captures["B2"], captures["A2"]
        )
        self.assertEqual(code, 0, report)
        claim = next(item for item in report["resolved_relative_claims"]
                     if item["metric"] == "metrics.mutation_nanos")
        self.assertEqual(claim["baseline_noise"], claim["remaining_headroom"])

    def test_matched_zero_posture_is_finite_and_nonzero_over_zero_regresses(self):
        captures = matched_captures()
        for capture_value in captures.values():
            set_metric(capture_value, "mutation_nanos", 0)
        report, code = PERF.compare_matched(
            captures["A1"], captures["B1"], captures["B2"], captures["A2"]
        )
        self.assertEqual(code, 0, report)
        set_metric(captures["B1"], "mutation_nanos", 1)
        report, code = PERF.compare_matched(
            captures["A1"], captures["B1"], captures["B2"], captures["A2"]
        )
        self.assertEqual(code, 1)
        claim = next(item for item in report["relative_regressions"]
                     if item["metric"] == "metrics.mutation_nanos")
        self.assertIsNone(claim["paired_ratios"]["A1_to_B1"])

    def test_matched_failed_or_relabelled_attempt_is_invalid_not_omitted(self):
        captures = matched_captures()
        captures["B2"]["status"] = "benchmark_failed"
        with self.assertRaisesRegex(ValueError, "failed/incomplete"):
            PERF.compare_matched(
                captures["A1"], captures["B1"], captures["B2"], captures["A2"]
            )
        captures = matched_captures()
        captures["B2"]["matched_capture"]["slot"] = "B1"
        with self.assertRaisesRegex(ValueError, "sequence index|expected matched slot"):
            PERF.compare_matched(
                captures["A1"], captures["B1"], captures["B2"], captures["A2"]
            )
        captures = matched_captures()
        captures["B2"]["matched_capture"]["set_id"] = "selected-other-set"
        with self.assertRaisesRegex(ValueError, "different sets"):
            PERF.compare_matched(
                captures["A1"], captures["B1"], captures["B2"], captures["A2"]
            )
        captures = matched_captures()
        captures["B1"]["matched_capture"]["started_unix_nanos"] = 4
        with self.assertRaisesRegex(ValueError, "capture start times"):
            PERF.compare_matched(
                captures["A1"], captures["B1"], captures["B2"], captures["A2"]
            )

    def test_matched_absolute_debt_remains_rejected_after_relative_resolution(self):
        captures = {slot: capture(access_value=1, slot=slot) for slot in PERF.CAPTURE_ORDER}
        report, code = PERF.compare_matched(
            captures["A1"], captures["B1"], captures["B2"], captures["A2"]
        )
        self.assertEqual(code, 4, report)
        self.assertEqual(report["relative_verdict"], "resolved_within_relative_budgets")
        self.assertEqual(report["absolute_verdict"], "violation")


if __name__ == "__main__":
    unittest.main()
