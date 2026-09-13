use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::measurement_capture::{PerfCaseContract, PerfCaseSummary};
use super::measurement_protocol::PerfTimingPolicy;
use super::regression_budgets::{
    ALLOCATION, MEDIAN_ONLY, PHASE, STRICT_MAX, STRICT_MEDIAN, STRICT_P95,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PerfBaselineFile {
    version: u32,
    #[serde(default)]
    environment: Option<PerfEnvironmentFingerprint>,
    cases: BTreeMap<String, PerfCaseSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct PerfEnvironmentFingerprint {
    target_os: String,
    target_arch: String,
    profile: String,
    toolchain: Option<String>,
    processor_identifier: Option<String>,
}

pub(super) fn certify_against_baseline(contract: PerfCaseContract<'_>, summary: &PerfCaseSummary) {
    if super::measurement_output::capture_requested() {
        // Explicit capture is measurement-only. The runner compares only after
        // every profile and its workload assertions have completed successfully.
        return;
    }
    let key = baseline_case_key(&summary.suite, &summary.profile, &summary.executor);
    let mut baseline = load_baseline_file();
    if update_perf_baseline() {
        baseline.version = 2;
        baseline.environment = Some(current_environment_fingerprint());
        baseline.cases.insert(key, summary.clone());
        write_baseline_file(&baseline);
        return;
    }

    let expected = baseline
        .cases
        .get(&key)
        .unwrap_or_else(|| panic!("missing perf baseline case for {key}"));
    assert_frozen_sample_count(&key, summary.sample_count, expected.sample_count);

    if baseline.environment.as_ref() == Some(&current_environment_fingerprint()) {
        match contract.timing_policy {
            PerfTimingPolicy::StrictHeavy => {
                assert_perf_regression_budget(
                    "elapsed median",
                    summary.elapsed_micros.median,
                    expected.elapsed_micros.median,
                    STRICT_MEDIAN,
                );
                assert_perf_regression_budget(
                    "elapsed p95",
                    summary.elapsed_micros.p95,
                    expected.elapsed_micros.p95,
                    STRICT_P95,
                );
                assert_perf_regression_budget(
                    "elapsed max",
                    summary.elapsed_micros.max,
                    expected.elapsed_micros.max,
                    STRICT_MAX,
                );
            }
            PerfTimingPolicy::MedianOnly => {
                assert_perf_regression_budget(
                    "elapsed median",
                    summary.elapsed_micros.median,
                    expected.elapsed_micros.median,
                    MEDIAN_ONLY,
                );
            }
            PerfTimingPolicy::StructuralOnly => {}
        }
    }

    for (counter, maximum) in contract.access_counter_maxima {
        let observed = summary
            .access_counters
            .get(*counter)
            .unwrap_or_else(|| panic!("missing summarized access counter {counter}"))
            .p95;
        assert!(
            observed <= *maximum,
            "access counter {} exceeded allowed maximum: observed {} > {}",
            counter,
            observed,
            maximum
        );
    }

    assert_perf_regression_budget(
        "allocated bytes median",
        summary.allocated_bytes.median,
        expected.allocated_bytes.median,
        ALLOCATION,
    );
    assert_perf_regression_budget(
        "allocated bytes max",
        summary.allocated_bytes.max,
        expected.allocated_bytes.max,
        ALLOCATION,
    );
    if let (Some(observed), Some(expected)) = (summary.allocation_calls, expected.allocation_calls)
    {
        assert_perf_regression_budget(
            "allocation calls median",
            observed.median,
            expected.median,
            ALLOCATION,
        );
        assert_perf_regression_budget(
            "allocation calls max",
            observed.max,
            expected.max,
            ALLOCATION,
        );
    }
    assert_perf_regression_budget(
        "end live bytes median",
        summary.end_live_bytes.median,
        expected.end_live_bytes.median,
        ALLOCATION,
    );
    assert_perf_regression_budget(
        "end live bytes max",
        summary.end_live_bytes.max,
        expected.end_live_bytes.max,
        ALLOCATION,
    );

    for (counter, observed) in &summary.access_counters {
        let expected = expected
            .access_counters
            .get(counter)
            .unwrap_or_else(|| panic!("missing baseline access counter {counter} for {key}"));
        assert!(
            observed.max <= expected.max,
            "access counter {counter} regressed for {key}: observed max {} > baseline max {}",
            observed.max,
            expected.max
        );
    }

    if !matches!(contract.timing_policy, PerfTimingPolicy::StructuralOnly) {
        for phase_metric in contract.phase_metrics {
            let observed = summary.phase_metrics.get(*phase_metric).unwrap_or_else(|| {
                panic!("missing observed phase metric {phase_metric} for {key}")
            });
            let expected = expected
                .phase_metrics
                .get(*phase_metric)
                .unwrap_or_else(|| {
                    panic!("missing baseline phase metric {phase_metric} for {key}")
                });
            assert_perf_regression_budget(
                &format!("phase metric {phase_metric} median"),
                observed.median,
                expected.median,
                PHASE,
            );
        }
    }

    for scoped_metric in contract.scoped_allocation_metrics {
        let observed = summary
            .scoped_allocation_metrics
            .get(*scoped_metric)
            .unwrap_or_else(|| {
                panic!("missing observed scoped allocation {scoped_metric} for {key}")
            });
        let expected = expected
            .scoped_allocation_metrics
            .get(*scoped_metric)
            .unwrap_or_else(|| {
                panic!("missing baseline scoped allocation {scoped_metric} for {key}")
            });
        assert_perf_regression_budget(
            &format!("scoped allocation {scoped_metric} median"),
            observed.median,
            expected.median,
            ALLOCATION,
        );
        assert_perf_regression_budget(
            &format!("scoped allocation {scoped_metric} max"),
            observed.max,
            expected.max,
            ALLOCATION,
        );
    }
}

fn assert_frozen_sample_count(key: &str, observed: usize, expected: usize) {
    assert_eq!(
        observed, expected,
        "sample count changed for {key}; stale golden evidence cannot certify the frozen protocol"
    );
}

fn assert_perf_regression_budget(label: &str, observed: u128, expected: u128, tolerance: f64) {
    let allowed = ((expected as f64) * tolerance).ceil() as u128;
    assert!(
        observed <= allowed,
        "{label} regressed: observed {observed} exceeds allowed {allowed} from baseline {expected}"
    );
}

fn baseline_case_key(suite: &str, profile: &str, executor: &str) -> String {
    format!("{suite}|{profile}|{executor}")
}

fn baseline_file_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("tests")
        .join("performance_baseline.json")
}

fn load_baseline_file() -> PerfBaselineFile {
    let path = baseline_file_path();
    let raw = fs::read_to_string(&path).unwrap_or_else(|err| {
        panic!(
            "failed to read perf baseline file {}: {err}",
            path.display()
        )
    });
    serde_json::from_str(&raw).unwrap_or_else(|err| {
        panic!(
            "failed to deserialize perf baseline file {}: {err}",
            path.display()
        )
    })
}

fn write_baseline_file(baseline: &PerfBaselineFile) {
    let path = baseline_file_path();
    let raw = serde_json::to_string_pretty(baseline).expect("perf baseline file should serialize");
    fs::write(&path, raw).unwrap_or_else(|err| {
        panic!(
            "failed to write perf baseline file {}: {err}",
            path.display()
        )
    });
}

fn update_perf_baseline() -> bool {
    env::var("WORTH_SIGNAL_UPDATE_PERF_BASELINE")
        .ok()
        .map(|value| {
            let value = value.trim().to_ascii_lowercase();
            value == "1" || value == "true" || value == "yes"
        })
        .unwrap_or(false)
}

fn current_environment_fingerprint() -> PerfEnvironmentFingerprint {
    PerfEnvironmentFingerprint {
        target_os: env::consts::OS.to_string(),
        target_arch: env::consts::ARCH.to_string(),
        profile: env::var("PROFILE").unwrap_or_else(|_| "unknown".to_string()),
        toolchain: env::var("RUSTUP_TOOLCHAIN").ok(),
        processor_identifier: env::var("PROCESSOR_IDENTIFIER").ok(),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{json, Value};

    use super::PerfCaseSummary;

    #[test]
    fn stale_sample_count_cannot_certify_the_frozen_protocol() {
        assert!(std::panic::catch_unwind(|| {
            super::assert_frozen_sample_count("chain|balanced|serial", 21, 7)
        })
        .is_err());
    }

    #[test]
    fn old_peak_key_is_decode_only_alias_for_end_live() {
        let checked = super::load_baseline_file();
        let checked_case = checked.cases.values().next().expect("checked perf case");
        assert!(checked_case.end_live_bytes.max > 0);

        let legacy = decode_summary(summary_json("peak_live_bytes", 41));
        assert_eq!(legacy.end_live_bytes.median, 42);
        assert_honest_encoding(&legacy, 42);

        let honest = decode_summary(summary_json("end_live_bytes", 81));
        assert_eq!(honest.end_live_bytes.median, 82);
        assert_honest_encoding(&honest, 82);

        let mut ambiguous = summary_json("end_live_bytes", 81);
        ambiguous["peak_live_bytes"] = numeric_summary(41);
        assert!(serde_json::from_value::<PerfCaseSummary>(ambiguous).is_err());
    }

    fn decode_summary(value: Value) -> PerfCaseSummary {
        serde_json::from_value(value).expect("valid performance summary")
    }

    fn assert_honest_encoding(summary: &PerfCaseSummary, expected_median: u128) {
        let encoded = serde_json::to_value(summary).unwrap();
        assert_eq!(
            encoded["end_live_bytes"]["median"].as_u64(),
            Some(expected_median as u64)
        );
        assert!(encoded.get("peak_live_bytes").is_none());
        assert!(encoded.get("legacy_end_live_bytes").is_none());
    }

    fn summary_json(end_live_key: &str, start: u128) -> Value {
        let mut summary = json!({
            "suite": "directional_contract",
            "profile": "balanced",
            "executor": "serial",
            "sample_count": 5,
            "elapsed_micros": numeric_summary(1),
            "allocation_calls": numeric_summary(11),
            "allocated_bytes": numeric_summary(21),
            "access_counters": {},
            "phase_metrics": {},
            "scoped_allocation_metrics": {},
        });
        summary[end_live_key] = numeric_summary(start);
        summary
    }

    fn numeric_summary(start: u128) -> Value {
        json!({
            "min": start,
            "median": start + 1,
            "p95": start + 2,
            "p99": start + 3,
            "max": start + 4,
        })
    }
}
