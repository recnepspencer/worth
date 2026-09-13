"""Frozen Signal whole-family measurement artifact and capture contract."""

import json
import math
import os
from pathlib import Path
import platform
import subprocess
import tomllib


CAPTURE_VERSION = 2
MAX_U128 = (1 << 128) - 1
PROTOCOL_ID = "signal-complete-family-abba-v2"
FAMILY = "tests::performance_profiles::"
CAPTURE_ORDER = ("A1", "B1", "B2", "A2")
BASELINE_SLOTS = ("A1", "A2")
CANDIDATE_SLOTS = ("B1", "B2")
PAIRINGS = (("A1", "B1"), ("A2", "B2"))
TEST_SUFFIXES = (
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
TESTS = [FAMILY + name for name in TEST_SUFFIXES]
CASES = [
    ("chain_10k_bootstrap", "balanced"),
    ("dependency_reconciliation_rotating_window", "balanced"),
    ("dependency_reconciliation_stable_shape_staged", "balanced"),
    ("dependency_reconciliation_rotating_window_staged", "balanced"),
    ("fintech_mixed_fanout", "operational"),
    ("fintech_mixed_fanout", "development"),
    ("fintech_mixed_fanout", "forensic"),
    ("harness_observability_profile", "development"),
    ("harness_observability_profile", "forensic"),
    ("suppression_wide_fanout", "balanced"),
    ("topology_rewiring_churn", "balanced"),
    ("topology_rewiring_rotating_window", "balanced"),
]
CONTRACTS = {
    ("chain_10k_bootstrap", "balanced"): ("median_only", 21, 4,
        ["build_nanos", "bootstrap_plan_nanos", "bootstrap_execute_nanos", "push_nanos"],
        [["materialized_entry_reads", 0], ["materialized_entry_writes", 0]]),
    ("dependency_reconciliation_rotating_window", "balanced"): ("structural_only", 3, 0,
        ["reconcile_loop_nanos", "dependency_reconcile_nanos", "snapshot_batch_commit_nanos"], []),
    ("dependency_reconciliation_stable_shape_staged", "balanced"): ("structural_only", 3, 0,
        ["planning_nanos", "report_stage_precompute_nanos", "report_stage_apply_nanos",
         "report_semantic_finalize_nanos", "dependency_reconcile_nanos", "snapshot_batch_commit_nanos"], []),
    ("dependency_reconciliation_rotating_window_staged", "balanced"): ("structural_only", 3, 0,
        ["planning_nanos", "report_stage_precompute_nanos", "report_stage_apply_nanos",
         "report_semantic_finalize_nanos", "dependency_reconcile_nanos", "snapshot_batch_commit_nanos"], []),
    ("fintech_mixed_fanout", "operational"): ("strict_heavy", 21, 4,
        ["read_before_nanos", "mutation_nanos", "read_after_nanos"], []),
    ("fintech_mixed_fanout", "development"): ("median_only", 21, 4,
        ["read_before_nanos", "mutation_nanos", "read_after_nanos"], []),
    ("fintech_mixed_fanout", "forensic"): ("median_only", 21, 4,
        ["read_before_nanos", "mutation_nanos", "read_after_nanos"], []),
    ("harness_observability_profile", "development"): ("structural_only", 3, 0,
        ["observe_loop_nanos"], []),
    ("harness_observability_profile", "forensic"): ("structural_only", 3, 0,
        ["observe_loop_nanos"], []),
    ("suppression_wide_fanout", "balanced"): ("median_only", 21, 4,
        ["leaf_reread_nanos", "stage_execution_nanos"],
        [["materialized_entry_reads", 0], ["materialized_entry_writes", 0]]),
    ("topology_rewiring_churn", "balanced"): ("median_only", 21, 4, ["rewire_loop_nanos"],
        [["materialized_entry_reads", 0], ["materialized_entry_writes", 0],
         ["runtime_artifact_state_reads", 0], ["runtime_artifact_warm_reads", 0]]),
    ("topology_rewiring_rotating_window", "balanced"): ("median_only", 21, 4, ["rewire_loop_nanos"],
        [["materialized_entry_reads", 0], ["materialized_entry_writes", 0],
         ["runtime_artifact_state_reads", 0], ["runtime_artifact_warm_reads", 0]]),
}
SCOPED_ALLOCATION_METRICS = {
    (suite, profile): [] for suite, profile in CASES
}
SCOPED_ALLOCATION_METRICS[("chain_10k_bootstrap", "balanced")] = [
    "push_scoped_allocation_calls", "push_scoped_requested_bytes"
]
WORKLOAD_WARMUPS = {(suite, profile): "none" for suite, profile in CASES}
WORKLOAD_WARMUPS.update({
    ("chain_10k_bootstrap", "balanced"): "once: build-plan-execute prime",
    ("fintech_mixed_fanout", "operational"): "once per profile: read-mutate-read prime",
    ("fintech_mixed_fanout", "development"): "once per profile: read-mutate-read prime",
    ("fintech_mixed_fanout", "forensic"): "once per profile: read-mutate-read prime",
    ("topology_rewiring_churn", "balanced"): "once: churn-rewire prime",
    ("topology_rewiring_rotating_window", "balanced"): "once: window-rewire prime",
})
PROBES = ("ordinary", "peak")
PEAK = "metrics.allocation_metrics.peak_live_bytes"
ACCESS_COUNTERS = (
    "materialized_entry_reads", "materialized_entry_writes",
    "runtime_artifact_warm_reads", "runtime_artifact_state_reads",
    "retained_artifact_reads", "reconstructed_artifact_reads",
)
CASE_RESOLUTION = {
    "strict_heavy": (21, 4),
    "median_only": (21, 4),
    "structural_only": (3, 0),
}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, f"duplicate JSON key: {key}")
        result[key] = value
    return result


def finite_float(raw):
    value = float(raw)
    require(math.isfinite(value), f"invalid number: {raw}")
    return value


def exact_value(actual, expected):
    if type(actual) is not type(expected):
        return False
    if isinstance(expected, dict):
        return actual.keys() == expected.keys() and all(
            exact_value(actual[key], value) for key, value in expected.items())
    if isinstance(expected, list):
        return len(actual) == len(expected) and all(
            exact_value(left, right) for left, right in zip(actual, expected))
    return actual == expected


def decode(raw):
    return json.loads(raw, object_pairs_hook=unique_object,
                      parse_float=finite_float,
                      parse_constant=lambda value: require(False, f"invalid number: {value}"))


def new_output(path, roots):
    require(path.is_absolute(), "output/build paths must be absolute")
    resolved = path.resolve()
    require(not any(resolved.is_relative_to(root.resolve()) for root in roots),
            "output/build paths must be outside source")
    require(path.parent.is_dir(), "output parent must already exist")
    require(not path.exists(), f"refusing existing output: {path}")
    require(path.name != "performance_baseline.json", "cannot write a golden")
    return resolved


def flatten(value, prefix=""):
    if isinstance(value, dict):
        result = {}
        for key, child in value.items():
            require(isinstance(key, str) and "." not in key, "invalid metric key")
            result.update(flatten(child, f"{prefix}.{key}" if prefix else key))
        return result
    return {prefix: value}


def distribution(values):
    require(values and all(type(value) is int and 0 <= value <= MAX_U128 for value in values),
            "metrics must be nonnegative u128 integers")
    ordered = sorted(values)
    return {"min": ordered[0], "median": ordered[math.ceil(len(values) * .5) - 1],
            "p95": ordered[math.ceil(len(values) * .95) - 1],
            "p99": ordered[math.ceil(len(values) * .99) - 1], "max": ordered[-1]}


def summarize(case):
    rows = [flatten(sample) for sample in case["samples"]]
    require(rows and all(row.keys() == rows[0].keys() for row in rows),
            "missing or changing sample metric roster")
    return {key: distribution([row[key] for row in rows])
            for key, value in rows[0].items() if type(value) is int}


def expected_budgets(contract, probe):
    if probe == "peak":
        return {PEAK: {"median": 1.10, "max": 1.10}}
    budgets = {}
    if contract["timing_policy"] == "strict_heavy":
        budgets["elapsed_micros"] = {"median": 1.20, "p95": 1.25, "max": 1.35}
    elif contract["timing_policy"] == "median_only":
        budgets["elapsed_micros"] = {"median": 1.35}
    for metric in ("allocation_calls", "allocated_bytes", "end_live_bytes"):
        budgets[f"metrics.allocation_metrics.{metric}"] = {"median": 1.10, "max": 1.10}
    for metric in contract["scoped_allocation_metrics"]:
        budgets[f"metrics.{metric}"] = {"median": 1.10, "max": 1.10}
    for counter in ACCESS_COUNTERS:
        budgets[f"metrics.access_counters.{counter}"] = {"max": 1.0}
    if contract["timing_policy"] != "structural_only":
        for phase in contract["phase_metrics"]:
            budgets[f"metrics.{phase}"] = {"median": 1.25}
    return budgets


def matched_protocol():
    return {
        "id": PROTOCOL_ID,
        "capture_order": list(CAPTURE_ORDER),
        "baseline_slots": list(BASELINE_SLOTS),
        "candidate_slots": list(CANDIDATE_SLOTS),
        "pairings": [list(pair) for pair in PAIRINGS],
        "baseline_noise": "max(A1/A2,A2/A1)-1",
        "remaining_headroom": "min((budget-paired_ratio)/budget)",
        "resolution": "conservative repeatability rule; not a confidence interval",
        "attempt_policy": "all four slots required; no rerun or best-run selection inside a set",
    }


def case_protocol(policy):
    samples, warmups = CASE_RESOLUTION[policy]
    return {
        "id": PROTOCOL_ID,
        "capture_order": list(CAPTURE_ORDER),
        "sample_count": samples,
        "warmup_count": warmups,
        "repeatability_rule": "symmetric A/A noise must fit conservative paired budget headroom",
        "statistical_posture": "order statistics; not a confidence interval",
    }


def validate_cases(cases, probe):
    require(isinstance(cases, list) and len(cases) == len(CASES),
            "incomplete twelve-case roster")
    for case, (suite, profile) in zip(cases, CASES):
        contract = case["contract"]
        require((contract["suite"], contract["profile"], contract["executor"])
                == (suite, profile, "serial"), "wrong case roster/order/executor")
        require(case["probe"] == probe, "wrong allocation probe posture")
        policy, samples, warmups, phases, maxima = CONTRACTS[(suite, profile)]
        require(exact_value(case["measurement_protocol"], case_protocol(policy)),
                "wrong Rust measurement protocol")
        require(exact_value(case["measurement_protocol"]["sample_count"], samples)
                and exact_value(case["measurement_protocol"]["warmup_count"], warmups),
                "protocol/case resolution mismatch")
        require(exact_value(case["sample_count"], samples) and len(case["samples"]) == samples,
                "wrong or missing sample count")
        require(exact_value(case["warmup_count"], warmups), "wrong warmup count")
        require(case["workload_warmup"] == WORKLOAD_WARMUPS[(suite, profile)],
                "wrong workload Once warmup posture")
        require(contract["timing_policy"] == policy, "wrong timing policy")
        require(contract["phase_metrics"] == phases, "wrong phase metric roster/order")
        require(contract["scoped_allocation_metrics"]
                == SCOPED_ALLOCATION_METRICS[(suite, profile)],
                "wrong scoped allocation metric roster/order")
        require(exact_value(contract["access_counter_maxima"], maxima),
                "wrong absolute contract roster")
        summaries = summarize(case)
        require(summaries["elapsed_micros"]["min"] > 0, "zero elapsed sample")
        for name in ("allocation_calls", "allocated_bytes", "deallocation_calls",
                     "deallocated_bytes", "live_bytes", "end_live_bytes"):
            require(f"metrics.allocation_metrics.{name}" in summaries, f"missing {name}")
        for phase in contract["phase_metrics"]:
            require(f"metrics.{phase}" in summaries, f"missing phase: {phase}")
        require(exact_value(case["relative_budgets"], expected_budgets(contract, probe)),
                "wrong relative budget authority")
        for metric, budgets in case["relative_budgets"].items():
            require(metric in summaries, f"missing budget metric: {metric}")
            require(budgets and all(stat in ("median", "p95", "max")
                    and type(ratio) in (float, int) and math.isfinite(ratio) and ratio >= 1
                    for stat, ratio in budgets.items()), "invalid budgets")
        for row in case["samples"]:
            allocation = row["metrics"]["allocation_metrics"]
            require(allocation["end_live_bytes"] == allocation["live_bytes"],
                    "legacy end-live compatibility mismatch")
            if probe == "ordinary":
                require(allocation["peak_live_bytes"] is None
                        and allocation["peak_live_status"].startswith("unavailable:"),
                        "ordinary peak must be explicitly unavailable")
            else:
                require(PEAK in summaries
                        and allocation["peak_live_status"].startswith(
                            "measured-group requested object high-water"),
                        "missing genuine peak")
                require(summaries[PEAK]["min"] > 0, "genuine peak tracker recorded zero")
    return cases


def validate_capture(data, expected_slot=None):
    require(exact_value(data["version"], CAPTURE_VERSION) and data["status"] == "captured",
            "failed/incomplete capture")
    require(exact_value(data["measurement_protocol"], matched_protocol()),
            "wrong matched protocol")
    matched = data["matched_capture"]
    require(isinstance(matched["set_id"], str) and matched["set_id"].strip(),
            "missing matched set id")
    require(matched["slot"] in CAPTURE_ORDER, "invalid matched capture slot")
    require(exact_value(matched["sequence_index"], CAPTURE_ORDER.index(matched["slot"])),
            "wrong matched capture sequence index")
    require(type(matched["started_unix_nanos"]) is int and matched["started_unix_nanos"] > 0,
            "invalid matched capture start time")
    if expected_slot is not None:
        require(matched["slot"] == expected_slot, f"expected matched slot {expected_slot}")
    require(data["test_roster"] == TESTS, "wrong test roster")
    require(exact_value(data["configuration"], configuration()),
            "wrong release/configuration posture")
    validate_environment(data["environment"])
    for probe in PROBES:
        require(exact_value(data["commands"][probe]["returncode"], 0),
                f"{probe} benchmark failed")
        validate_command(data["commands"][probe]["argv"], probe)
        validate_cases(data["cases"][probe], probe)
    ordinary_command = data["commands"]["ordinary"]["argv"]
    peak_command = data["commands"]["peak"]["argv"]
    require(ordinary_command[5:8:2] == peak_command[5:8:2],
            "ordinary/peak source or target path differs")
    for ordinary, peak in zip(data["cases"]["ordinary"], data["cases"]["peak"]):
        require(case_posture(ordinary) == case_posture(peak),
                "ordinary/peak contracts differ")
        require(sample_metric_roster(ordinary) == sample_metric_roster(peak),
                "ordinary/peak sample metric roster differs")
    return data


def validate_environment(environment):
    required = {"os", "arch", "cpu", "host", "logical_cpus", "variables",
                "rustc", "cargo", "workspace_profiles", "cargo_config"}
    require(isinstance(environment, dict) and set(environment) == required,
            "missing or extra environment fields")
    for name in ("os", "arch", "cpu", "host", "rustc", "cargo", "cargo_config"):
        require(isinstance(environment[name], str) and environment[name].strip()
                and (name != "cpu" or environment[name].strip().lower() != "unknown"),
                f"invalid environment field: {name}")
    require(type(environment["logical_cpus"]) is int and environment["logical_cpus"] > 0,
            "invalid logical CPU count")
    require(isinstance(environment["variables"], dict)
            and all(isinstance(key, str) and isinstance(value, str)
                    for key, value in environment["variables"].items()),
            "invalid environment variables")
    require(isinstance(environment["workspace_profiles"], dict)
            and environment["workspace_profiles"], "missing workspace release profiles")


def case_config(case):
    return {key: case[key] for key in (
        "measurement_protocol", "contract", "sample_count", "warmup_count", "relative_budgets"
    )}


def case_posture(case):
    return {key: case[key] for key in (
        "measurement_protocol", "contract", "sample_count", "warmup_count", "workload_warmup"
    )}


def sample_metric_roster(case):
    return set(flatten(case["samples"][0]).keys())


def validate_command(argv, probe):
    require(isinstance(argv, list) and all(isinstance(value, str) for value in argv),
            "missing benchmark argv")
    feature = "profile-extended" + (",test-peak-allocation" if probe == "peak" else "")
    require(len(argv) == 20, "wrong benchmark argv length")
    manifest, target = Path(argv[5]), Path(argv[7])
    require(manifest.is_absolute() and manifest.name == "Cargo.toml", "invalid manifest path")
    require(target.is_absolute(), "target dir must be absolute")
    expected = ["cargo", "test", "--locked", "--release", "--manifest-path", str(manifest),
                "--target-dir", str(target), "-p", "worth-signal", "--lib", "--no-default-features",
                "--features", feature, FAMILY, "--", "--ignored", "--test-threads=1", "--nocapture",
                "--color=never"]
    require(argv == expected, "wrong exact cargo/libtest benchmark posture")


def configuration():
    resolutions = {
        policy: {"samples": resolution[0], "warmups": resolution[1]}
        for policy, resolution in CASE_RESOLUTION.items()
    }
    return {"profile": "release", "default_features": False, "features": ["profile-extended"],
            "peak_feature": "test-peak-allocation", "test_threads": 1, "ignored": True,
            "filter": FAMILY, "case_resolution": resolutions,
            "workload_warmups": [[suite, profile, WORKLOAD_WARMUPS[(suite, profile)]]
                                 for suite, profile in CASES]}


def environment(root):
    keys = ("RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "RUSTUP_TOOLCHAIN", "RUST_MIN_STACK",
            "RUST_LOG", "RAYON_NUM_THREADS")
    variables = {key: value for key, value in os.environ.items()
                 if key in keys or key.startswith(("CARGO_PROFILE_", "CARGO_BUILD_", "CARGO_TARGET_"))}
    require(not any(key.startswith("WORTH_SIGNAL_") for key in os.environ),
            "remove ambient WORTH_SIGNAL_* overrides before capture")
    cpu = platform.processor() or os.environ.get("PROCESSOR_IDENTIFIER", "")
    context = {"os": platform.platform(), "arch": platform.machine(), "cpu": cpu,
               "host": platform.node(), "logical_cpus": os.cpu_count(), "variables": variables,
               "rustc": subprocess.check_output(["rustc", "-vV"], cwd=root, text=True),
               "cargo": subprocess.check_output(["cargo", "-V"], cwd=root, text=True),
               "workspace_profiles": tomllib.loads((root / "Cargo.toml").read_text())["profile"],
               "cargo_config": (root / ".cargo/config.toml").read_text()}
    validate_environment(context)
    return context
