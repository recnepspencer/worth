"""Synthetic complete-family fixtures for Signal comparator tests."""

from pathlib import Path

import compare_signal_performance as PERF

SYNTHETIC_ROOT = Path.cwd().resolve() / "signal-performance-fixture"


def sample(contract, probe, value=100, access_value=0):
    allocation = {
        "allocation_calls": value,
        "deallocation_calls": value,
        "allocated_bytes": value,
        "deallocated_bytes": value,
        "live_bytes": value,
        "end_live_bytes": value,
        "peak_live_bytes": None if probe == "ordinary" else value,
        "peak_live_status": (
            "unavailable: ordinary timing uses the unwrapped stats allocator"
            if probe == "ordinary"
            else "measured-group requested object high-water; instrumented realloc allocates/copies/frees"
        ),
    }
    metrics = {
        "allocation_metrics": allocation,
        "access_counters": {name: access_value for name in PERF.ACCESS_COUNTERS},
    }
    metrics.update({phase: value for phase in contract["phase_metrics"]})
    metrics.update({metric: value for metric in contract["scoped_allocation_metrics"]})
    return {"elapsed_micros": value, "metrics": metrics}


def capture(value=100, access_value=0, slot="A1", set_id="test-set"):
    cases = {}
    for probe in PERF.PROBES:
        records = []
        for suite, profile in PERF.CASES:
            policy, sample_count, warmups, phases, maxima = PERF.CONTRACTS[(suite, profile)]
            contract = {
                "suite": suite,
                "profile": profile,
                "executor": "serial",
                "timing_policy": policy,
                "phase_metrics": phases,
                "scoped_allocation_metrics": PERF.SCOPED_ALLOCATION_METRICS[(suite, profile)],
                "access_counter_maxima": maxima,
            }
            records.append({
                "measurement_protocol": PERF.case_protocol(policy),
                "contract": contract,
                "probe": probe,
                "sample_count": sample_count,
                "warmup_count": warmups,
                "workload_warmup": PERF.WORKLOAD_WARMUPS[(suite, profile)],
                "relative_budgets": PERF.expected_budgets(contract, probe),
                "samples": [sample(contract, probe, value, access_value)
                            for _ in range(sample_count)],
            })
        cases[probe] = records
    environment = {
        "os": "test-os", "arch": "test-arch", "cpu": "test-cpu",
        "host": "controlled-test-host", "logical_cpus": 1, "variables": {},
        "rustc": "rustc test", "cargo": "cargo test",
        "workspace_profiles": {"release": {"opt-level": 3}}, "cargo_config": "[build]",
    }
    commands = {}
    for probe in PERF.PROBES:
        feature = "profile-extended" + (",test-peak-allocation" if probe == "peak" else "")
        commands[probe] = {"returncode": 0, "argv": [
            "cargo", "test", "--locked", "--release", "--manifest-path",
            str(SYNTHETIC_ROOT / "baseline/Cargo.toml"), "--target-dir",
            str(SYNTHETIC_ROOT / "target"), "-p", "worth-signal", "--lib",
            "--no-default-features", "--features", feature, PERF.FAMILY, "--", "--ignored",
            "--test-threads=1", "--nocapture", "--color=never",
        ]}
    return {
        "version": PERF.CAPTURE_VERSION,
        "status": "captured",
        "measurement_protocol": PERF.matched_protocol(),
        "matched_capture": {
            "set_id": set_id,
            "slot": slot,
            "sequence_index": PERF.CAPTURE_ORDER.index(slot),
            "started_unix_nanos": PERF.CAPTURE_ORDER.index(slot) + 1,
        },
        "configuration": PERF.configuration(),
        "environment": environment,
        "test_roster": PERF.TESTS,
        "commands": commands,
        "cases": cases,
    }


def matched_captures():
    return {slot: capture(slot=slot) for slot in PERF.CAPTURE_ORDER}


def set_metric(capture_value, metric, value):
    _set_case_metric(capture_value, "fintech_mixed_fanout", "operational", metric, value)


def set_chain_metric(capture_value, metric, value):
    _set_case_metric(capture_value, "chain_10k_bootstrap", "balanced", metric, value)


def _set_case_metric(capture_value, suite, profile, metric, value):
    for case in capture_value["cases"]["ordinary"]:
        if (case["contract"]["suite"], case["contract"]["profile"]) == (suite, profile):
            for row in case["samples"]:
                row["metrics"][metric] = value
            return
    raise AssertionError(f"{suite}|{profile} case missing")
