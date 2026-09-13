use std::collections::BTreeMap;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use stats_alloc::{Region, StatsAlloc, INSTRUMENTED_SYSTEM};

use super::measurement_protocol::{case_resolution, PerfTimingPolicy};
use super::regression_budgets::ACCESS_COUNTERS;

static PERF_ALLOC_LOCK: Mutex<()> = Mutex::new(());

#[global_allocator]
#[cfg(not(feature = "test-peak-allocation"))]
static GLOBAL_ALLOCATOR: &StatsAlloc<std::alloc::System> = &INSTRUMENTED_SYSTEM;

#[global_allocator]
#[cfg(feature = "test-peak-allocation")]
static GLOBAL_ALLOCATOR: tracking_allocator::Allocator<&StatsAlloc<std::alloc::System>> =
    tracking_allocator::Allocator::from_allocator(&INSTRUMENTED_SYSTEM);

#[derive(Debug, Clone, Copy)]
struct AllocationStats {
    allocation_calls: u64,
    deallocation_calls: u64,
    pub(super) allocated_bytes: usize,
    deallocated_bytes: usize,
    live_bytes: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PerfMeasurement {
    pub elapsed_micros: u128,
    pub metrics: serde_json::Value,
}

impl PerfMeasurement {
    pub(crate) fn new(elapsed_micros: u128, metrics: serde_json::Value) -> Self {
        Self {
            elapsed_micros,
            metrics,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
pub(crate) struct PerfCaseContract<'a> {
    pub suite: &'a str,
    pub profile: &'a str,
    pub executor: &'a str,
    pub timing_policy: PerfTimingPolicy,
    pub phase_metrics: &'a [&'a str],
    pub scoped_allocation_metrics: &'a [&'a str],
    pub access_counter_maxima: &'a [(&'a str, u128)],
}

impl<'a> PerfCaseContract<'a> {
    pub(crate) const fn new(
        suite: &'a str,
        profile: &'a str,
        executor: &'a str,
        timing_policy: PerfTimingPolicy,
        phase_metrics: &'a [&'a str],
        scoped_allocation_metrics: &'a [&'a str],
        access_counter_maxima: &'a [(&'a str, u128)],
    ) -> Self {
        Self {
            suite,
            profile,
            executor,
            timing_policy,
            phase_metrics,
            scoped_allocation_metrics,
            access_counter_maxima,
        }
    }
}

#[derive(Debug, Serialize)]
struct PerfSampleRecord<'a> {
    kind: &'static str,
    pub(super) suite: &'a str,
    pub(super) profile: &'a str,
    pub(super) executor: &'a str,
    sample_index: usize,
    pub(super) elapsed_micros: u128,
    metrics: &'a serde_json::Value,
}

#[derive(Debug, Serialize)]
struct PerfSummaryRecord<'a> {
    kind: &'static str,
    pub(super) suite: &'a str,
    pub(super) profile: &'a str,
    pub(super) executor: &'a str,
    pub(super) sample_count: usize,
    min_elapsed_micros: u128,
    median_elapsed_micros: u128,
    p95_elapsed_micros: u128,
    p99_elapsed_micros: u128,
    max_elapsed_micros: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct PerfCaseSummary {
    pub(super) suite: String,
    pub(super) profile: String,
    pub(super) executor: String,
    pub(super) sample_count: usize,
    pub(super) elapsed_micros: NumericSummary,
    #[serde(default)]
    pub(super) allocation_calls: Option<NumericSummary>,
    pub(super) allocated_bytes: NumericSummary,
    #[serde(alias = "peak_live_bytes")]
    pub(super) end_live_bytes: NumericSummary,
    pub(super) access_counters: BTreeMap<String, NumericSummary>,
    #[serde(default)]
    pub(super) phase_metrics: BTreeMap<String, NumericSummary>,
    #[serde(default)]
    pub(super) scoped_allocation_metrics: BTreeMap<String, NumericSummary>,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub(super) struct NumericSummary {
    pub(super) min: u128,
    pub(super) median: u128,
    pub(super) p95: u128,
    pub(super) p99: u128,
    pub(super) max: u128,
}

pub(crate) fn capture_perf_samples<F>(
    contract: PerfCaseContract<'_>,
    mut measure: F,
) -> Vec<PerfMeasurement>
where
    F: FnMut() -> PerfMeasurement,
{
    let sample_count = perf_sample_count(contract.timing_policy);
    super::measurement_output::validate_capture_posture();
    let mut samples = Vec::with_capacity(sample_count);
    let _alloc_guard = PERF_ALLOC_LOCK
        .lock()
        .expect("perf allocation instrumentation lock should not be poisoned");

    // Prime allocator pages, graph interners, and branch-local caches before samples.
    for _ in 0..perf_warmup_count(contract.timing_policy) {
        let _ = measure();
    }

    for sample_index in 0..sample_count {
        let access_before = crate::data::access_counters::snapshot();
        #[cfg(not(feature = "test-peak-allocation"))]
        let region = Region::new(GLOBAL_ALLOCATOR);
        #[cfg(feature = "test-peak-allocation")]
        let region = Region::new(&INSTRUMENTED_SYSTEM);
        #[cfg(not(feature = "test-peak-allocation"))]
        let (mut measurement, measured_peak) = (measure(), None);
        #[cfg(feature = "test-peak-allocation")]
        let (mut measurement, measured_peak) = super::peak_allocation::measure(&mut measure);
        let access_after = crate::data::access_counters::snapshot();
        attach_allocation_stats(
            &mut measurement.metrics,
            snapshot_allocation_stats(&region),
            measured_peak,
        );
        attach_access_counters(
            &mut measurement.metrics,
            access_after.delta_since(access_before),
        );
        emit(&PerfSampleRecord {
            kind: "sample",
            suite: contract.suite,
            profile: contract.profile,
            executor: contract.executor,
            sample_index,
            elapsed_micros: measurement.elapsed_micros,
            metrics: &measurement.metrics,
        });
        samples.push(measurement);
    }

    let mut elapsed = samples
        .iter()
        .map(|sample| sample.elapsed_micros)
        .collect::<Vec<_>>();
    elapsed.sort_unstable();
    emit(&PerfSummaryRecord {
        kind: "summary",
        suite: contract.suite,
        profile: contract.profile,
        executor: contract.executor,
        sample_count,
        min_elapsed_micros: elapsed[0],
        median_elapsed_micros: percentile(&elapsed, 50),
        p95_elapsed_micros: percentile(&elapsed, 95),
        p99_elapsed_micros: percentile(&elapsed, 99),
        max_elapsed_micros: elapsed[elapsed.len() - 1],
    });

    samples
}

pub(super) fn summarize_perf_samples(
    contract: PerfCaseContract<'_>,
    samples: &[PerfMeasurement],
) -> PerfCaseSummary {
    let elapsed = samples
        .iter()
        .map(|sample| sample.elapsed_micros)
        .collect::<Vec<_>>();
    let allocated_bytes = samples
        .iter()
        .map(|sample| numeric_metric(&sample.metrics, &["allocation_metrics", "allocated_bytes"]))
        .collect::<Vec<_>>();
    let allocation_calls = samples
        .iter()
        .map(|sample| numeric_metric(&sample.metrics, &["allocation_metrics", "allocation_calls"]))
        .collect::<Vec<_>>();
    let end_live_bytes = samples
        .iter()
        .map(|sample| numeric_metric(&sample.metrics, &["allocation_metrics", "end_live_bytes"]))
        .collect::<Vec<_>>();

    let mut access_counters = BTreeMap::new();
    for key in ACCESS_COUNTERS {
        let values = samples
            .iter()
            .map(|sample| numeric_metric(&sample.metrics, &["access_counters", key]))
            .collect::<Vec<_>>();
        access_counters.insert(key.to_string(), summarize_u128(&values));
    }

    let mut phase_metrics = BTreeMap::new();
    for key in contract.phase_metrics {
        let values = samples
            .iter()
            .map(|sample| numeric_metric(&sample.metrics, &[key]))
            .collect::<Vec<_>>();
        phase_metrics.insert((*key).to_string(), summarize_u128(&values));
    }

    let mut scoped_allocation_metrics = BTreeMap::new();
    for key in contract.scoped_allocation_metrics {
        let values = samples
            .iter()
            .map(|sample| numeric_metric(&sample.metrics, &[key]))
            .collect::<Vec<_>>();
        scoped_allocation_metrics.insert((*key).to_string(), summarize_u128(&values));
    }

    let summary = PerfCaseSummary {
        suite: contract.suite.to_string(),
        profile: contract.profile.to_string(),
        executor: contract.executor.to_string(),
        sample_count: samples.len(),
        elapsed_micros: summarize_u128(&elapsed),
        allocation_calls: Some(summarize_u128(&allocation_calls)),
        allocated_bytes: summarize_u128(&allocated_bytes),
        end_live_bytes: summarize_u128(&end_live_bytes),
        access_counters,
        phase_metrics,
        scoped_allocation_metrics,
    };

    super::measurement_output::record_case(contract, samples);
    emit(&summary);
    summary
}

pub(super) fn perf_sample_count(timing_policy: PerfTimingPolicy) -> usize {
    case_resolution(timing_policy).sample_count()
}

pub(super) fn perf_warmup_count(timing_policy: PerfTimingPolicy) -> usize {
    case_resolution(timing_policy).warmup_count()
}

fn percentile(sorted: &[u128], percentile: usize) -> u128 {
    debug_assert!(!sorted.is_empty());
    debug_assert!(percentile <= 100);

    let rank = (sorted.len() * percentile).div_ceil(100).saturating_sub(1);
    sorted[rank]
}

fn emit(record: &impl Serialize) {
    eprintln!(
        "{}",
        serde_json::to_string(record).expect("perf record should serialize")
    );
}

fn numeric_metric(metrics: &serde_json::Value, path: &[&str]) -> u128 {
    let mut current = metrics;
    for segment in path {
        current = current
            .get(*segment)
            .unwrap_or_else(|| panic!("missing perf metric path {}", path.join(".")));
    }
    current
        .as_u64()
        .map(u128::from)
        .unwrap_or_else(|| panic!("non-numeric perf metric path {}", path.join(".")))
}

fn summarize_u128(values: &[u128]) -> NumericSummary {
    debug_assert!(!values.is_empty());
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    NumericSummary {
        min: sorted[0],
        median: percentile(&sorted, 50),
        p95: percentile(&sorted, 95),
        p99: percentile(&sorted, 99),
        max: sorted[sorted.len() - 1],
    }
}

fn snapshot_allocation_stats(region: &Region<'_, std::alloc::System>) -> AllocationStats {
    let stats = region.change();
    let live_bytes = stats
        .bytes_allocated
        .saturating_sub(stats.bytes_deallocated);
    AllocationStats {
        allocation_calls: stats.allocations as u64,
        deallocation_calls: stats.deallocations as u64,
        allocated_bytes: stats.bytes_allocated,
        deallocated_bytes: stats.bytes_deallocated,
        live_bytes,
    }
}

fn attach_allocation_stats(
    metrics: &mut serde_json::Value,
    stats: AllocationStats,
    measured_peak: Option<usize>,
) {
    let peak_status = if measured_peak.is_some() {
        "measured-group requested object high-water; instrumented realloc allocates/copies/frees"
    } else {
        "unavailable: ordinary timing uses the unwrapped stats allocator"
    };
    let allocation_metrics = serde_json::json!({
        "allocation_calls": stats.allocation_calls,
        "deallocation_calls": stats.deallocation_calls,
        "allocated_bytes": stats.allocated_bytes,
        "deallocated_bytes": stats.deallocated_bytes,
        "live_bytes": stats.live_bytes,
        "end_live_bytes": stats.live_bytes,
        "peak_live_bytes": measured_peak,
        "peak_live_status": peak_status,
    });

    match metrics {
        serde_json::Value::Object(map) => {
            map.insert("allocation_metrics".into(), allocation_metrics);
        }
        _ => {
            *metrics = serde_json::json!({
                "reported_metrics": metrics.clone(),
                "allocation_metrics": allocation_metrics,
            });
        }
    }
}

fn attach_access_counters(
    metrics: &mut serde_json::Value,
    counters: crate::data::access_counters::AccessCounterSnapshot,
) {
    let access_metrics = serde_json::json!(counters);

    match metrics {
        serde_json::Value::Object(map) => {
            map.insert("access_counters".into(), access_metrics);
        }
        _ => {
            *metrics = serde_json::json!({
                "reported_metrics": metrics.clone(),
                "access_counters": access_metrics,
            });
        }
    }
}
