use super::measurement_capture::PerfCaseContract;
use super::measurement_protocol::PerfTimingPolicy;
use serde_json::{json, Value};

pub(super) const ACCESS_COUNTERS: &[&str] = &[
    "materialized_entry_reads",
    "materialized_entry_writes",
    "runtime_artifact_warm_reads",
    "runtime_artifact_state_reads",
    "retained_artifact_reads",
    "reconstructed_artifact_reads",
];

// These are the existing golden budgets, also emitted for direct comparisons.
pub(super) const STRICT_MEDIAN: f64 = 1.20;
pub(super) const STRICT_P95: f64 = 1.25;
pub(super) const STRICT_MAX: f64 = 1.35;
pub(super) const MEDIAN_ONLY: f64 = 1.35;
pub(super) const ALLOCATION: f64 = 1.10;
pub(super) const PHASE: f64 = 1.25;

pub(super) fn relative_budgets(contract: PerfCaseContract<'_>, peak_probe: bool) -> Value {
    let mut budgets = json!({});
    if peak_probe {
        budgets["metrics.allocation_metrics.peak_live_bytes"] =
            json!({"median": ALLOCATION, "max": ALLOCATION});
        return budgets;
    }
    match contract.timing_policy {
        PerfTimingPolicy::StrictHeavy => {
            budgets["elapsed_micros"] = json!({
                "median": STRICT_MEDIAN, "p95": STRICT_P95, "max": STRICT_MAX,
            });
        }
        PerfTimingPolicy::MedianOnly => {
            budgets["elapsed_micros"] = json!({"median": MEDIAN_ONLY});
        }
        PerfTimingPolicy::StructuralOnly => {}
    }
    for metric in ["allocation_calls", "allocated_bytes", "end_live_bytes"] {
        budgets[format!("metrics.allocation_metrics.{metric}")] =
            json!({"median": ALLOCATION, "max": ALLOCATION});
    }
    for metric in contract.scoped_allocation_metrics {
        budgets[format!("metrics.{metric}")] = json!({"median": ALLOCATION, "max": ALLOCATION});
    }
    for counter in ACCESS_COUNTERS {
        budgets[format!("metrics.access_counters.{counter}")] = json!({"max": 1.0});
    }
    if !matches!(contract.timing_policy, PerfTimingPolicy::StructuralOnly) {
        for phase in contract.phase_metrics {
            budgets[format!("metrics.{phase}")] = json!({"median": PHASE});
        }
    }
    budgets
}
