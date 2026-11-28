#!/usr/bin/env python3
import argparse
import json
import math
import sys
from pathlib import Path

NOISY_CASES = {
    ("commit_delta_matrix", "single_partition_create_burst"),
    ("commit_delta_matrix", "persisted_single_entity_create"),
    ("commit_delta_matrix", "cross_partition_relation_burst"),
    ("chip_simulator_matrix", "flat_entity_step_batch_compile_window"),
    ("query_packet_matrix", "connectivity_traversal_cross_partition"),
    ("query_packet_matrix", "explicit_targets_cross_partition"),
    ("replay_recovery_matrix", "durable_replay_lineage_basis"),
    ("replay_recovery_matrix", "checkpoint_recover_suffix_replay"),
    ("snapshot_materialization_matrix", "version_read_view_historical"),
    ("workflow_matrix", "retention_release_reclaim_round_trip"),
}

BURSTY_CASES = {
    ("query_packet_matrix", "explicit_targets_cross_partition"),
    ("workflow_matrix", "trade_correction_analysis_round_trip"),
    ("workflow_matrix", "fintech_intraday_risk_branch_round_trip"),
    ("workflow_matrix", "fintech_trade_correction_audit_round_trip"),
    ("profile_matrix", "certification_core_zero_diagnostics_commit_query_round_trip"),
}

IO_BURSTY_CASES = {
    ("commit_delta_matrix", "persisted_single_entity_create"),
    ("durability_append_matrix", "append_canonical_envelope_fresh_store"),
    ("durability_append_matrix", "append_canonical_envelope_existing_segment"),
    ("replay_recovery_matrix", "durable_replay_lineage_basis"),
    ("rocketship_scale_matrix", "hundred_k_nodes_pseudorealistic_large_flat_entity_batch_wave"),
    ("rocketship_scale_matrix", "hundred_k_nodes_pseudorealistic_mixed_entity_relation_batch_wave"),
}

VOLATILE_CASES = {
    ("commit_delta_matrix", "single_partition_create_burst"),
    ("geometry_kernel_matrix", "topology_identity_survival_recovery_round_trip"),
    ("chip_simulator_matrix", "branch_rollback_compile_step_window"),
    ("index_parity_matrix", "entity_field_equals_warm_generation"),
    ("inspection_budget_matrix", "retention_commit_window"),
    ("sustained_load_matrix", "commit_query_churn_stability"),
    ("snapshot_materialization_matrix", "projection_entity_identity_surface"),
    ("workflow_matrix", "persisted_recovery_replay_round_trip"),
    ("profile_matrix", "certification_core_rich_commit_query_round_trip"),
    ("profile_matrix", "geometry_kernel_rich_commit_query_round_trip"),
}

EXTREME_VOLATILE_CASES = {
    ("chip_simulator_matrix", "event_wave_compile_churn_window"),
    ("chip_simulator_matrix", "event_wave_compile_churn_rich_diagnostics"),
}

TINY_CASES = {
    ("merge_lineage_matrix", "lineage_branch_divergence_breadth"),
    ("query_packet_matrix", "entity_kind_scan_partition_matrix"),
    ("retention_reclaim_matrix", "replay_pin_release_deleted_relation"),
    ("snapshot_materialization_matrix", "snapshot_read_view_current"),
}

MICRO_CASES = {
    ("retention_reclaim_matrix", "snapshot_release_to_reclaimable_entity"),
    ("workflow_matrix", "retention_release_reclaim_round_trip"),
}


def load_jsonl(path: Path):
    rows = []
    for raw in path.read_text(encoding="utf-8").splitlines():
        raw = raw.strip()
        if raw:
            rows.append(json.loads(raw))
    return rows


def row_key(row):
    return (row["suite"], row["case"], row.get("metric"))


def row_sort_key(key):
    suite, case, metric = key
    return (suite, case, "" if metric is None else metric)


def allowed(value, tolerance):
    return math.ceil(value * tolerance)


def profile_boundary_metric(metric):
    return metric in {
        "profile_execution_lane_code",
        "profile_diagnostics_boundary_code",
        "profile_matches_defaults",
    }


def case_tolerances(suite, case):
    if (suite, case) in MICRO_CASES:
        return {
            "elapsed_median": 3.00,
            "elapsed_max": 3.50,
            "metric_median": 3.00,
            "enforce_elapsed_max": True,
        }
    if (suite, case) in TINY_CASES:
        return {
            "elapsed_median": 2.25,
            "elapsed_max": 2.50,
            "metric_median": 2.00,
            "enforce_elapsed_max": True,
        }
    if (suite, case) in IO_BURSTY_CASES:
        return {
            "elapsed_median": 1.90,
            "elapsed_max": 2.50,
            "metric_median": 2.00,
            "enforce_elapsed_max": False,
        }
    if (suite, case) in EXTREME_VOLATILE_CASES:
        return {
            "elapsed_median": 2.50,
            "elapsed_max": 3.00,
            "metric_median": 3.00,
            "enforce_elapsed_max": False,
        }
    if (suite, case) in VOLATILE_CASES:
        return {
            "elapsed_median": 2.10,
            "elapsed_max": 2.50,
            "metric_median": 2.25,
            "enforce_elapsed_max": False,
        }
    if (suite, case) in BURSTY_CASES:
        return {
            "elapsed_median": 1.75,
            "elapsed_max": 2.10,
            "metric_median": 1.90,
            "enforce_elapsed_max": True,
        }
    if (suite, case) in NOISY_CASES:
        return {
            "elapsed_median": 1.50,
            "elapsed_max": 1.75,
            "metric_median": 1.75,
            "enforce_elapsed_max": True,
        }
    return {
        "elapsed_median": 1.20,
        "elapsed_max": 1.35,
        "metric_median": 1.25,
        "enforce_elapsed_max": True,
    }


def main():
    parser = argparse.ArgumentParser(description="Check worth-relational perf summary against a baseline")
    parser.add_argument("--baseline", required=True, help="Baseline summary JSONL")
    parser.add_argument("--current", required=True, help="Current summary JSONL")
    parser.add_argument("--elapsed-median-tolerance", type=float, default=1.20)
    parser.add_argument("--elapsed-max-tolerance", type=float, default=1.35)
    parser.add_argument("--metric-median-tolerance", type=float, default=1.25)
    args = parser.parse_args()

    baseline_rows = {row_key(row): row for row in load_jsonl(Path(args.baseline))}
    current_rows = {row_key(row): row for row in load_jsonl(Path(args.current))}

    failures = []

    missing = sorted(set(baseline_rows) - set(current_rows), key=row_sort_key)
    if missing:
        failures.extend(f"missing current summary row for {suite}/{case}/{metric or 'elapsed'}" for suite, case, metric in missing)

    new_rows = sorted(set(current_rows) - set(baseline_rows), key=row_sort_key)
    if new_rows:
        failures.extend(f"missing baseline summary row for {suite}/{case}/{metric or 'elapsed'}" for suite, case, metric in new_rows)

    for key in sorted(set(current_rows) & set(baseline_rows), key=lambda item: (item[0], item[1], item[2] or "")):
        baseline = baseline_rows[key]
        current = current_rows[key]
        suite, case, metric = key
        label = f"{suite}/{case}" + (f"/{metric}" if metric else "")

        if metric is None:
            baseline_median = baseline["median_elapsed_micros"]
            current_median = current["median_elapsed_micros"]
            tolerances = case_tolerances(suite, case)
            median_limit = allowed(baseline_median, tolerances["elapsed_median"])
            if current_median > median_limit:
                failures.append(
                    f"{label} median regressed: observed {current_median} > allowed {median_limit} from baseline {baseline_median}"
                )

            baseline_max = baseline["max_elapsed_micros"]
            current_max = current["max_elapsed_micros"]
            if tolerances["enforce_elapsed_max"]:
                max_limit = allowed(baseline_max, tolerances["elapsed_max"])
                if current_max > max_limit:
                    failures.append(
                        f"{label} max regressed: observed {current_max} > allowed {max_limit} from baseline {baseline_max}"
                    )
        else:
            baseline_median = baseline["median"]
            current_median = current["median"]
            if profile_boundary_metric(metric):
                if current_median != baseline_median:
                    failures.append(
                        f"{label} profile drift: observed {current_median} != baseline {baseline_median}"
                    )
                continue
            tolerances = case_tolerances(suite, case)
            median_limit = allowed(baseline_median, tolerances["metric_median"])
            if current_median > median_limit:
                failures.append(
                    f"{label} metric median regressed: observed {current_median} > allowed {median_limit} from baseline {baseline_median}"
                )

    if failures:
        print("[relational-perf-baseline] FAIL")
        for failure in failures:
            print(f"- {failure}")
        sys.exit(1)

    print("[relational-perf-baseline] PASS")


if __name__ == "__main__":
    main()
