mod measurement_capture;
mod measurement_output;
mod measurement_protocol;
#[cfg(feature = "test-peak-allocation")]
mod peak_allocation;
mod profile_comparison;
mod regression_budgets;
mod workload_construction;

pub(crate) use measurement_capture::{PerfCaseContract, PerfMeasurement};
pub(crate) use measurement_protocol::PerfTimingPolicy;
pub(crate) use workload_construction::{build_chain_graph, with_perf_topology_asserts_disabled};

pub(crate) fn capture_and_certify_perf_samples<F>(
    contract: PerfCaseContract<'_>,
    measure: F,
) -> Vec<PerfMeasurement>
where
    F: FnMut() -> PerfMeasurement,
{
    let samples = measurement_capture::capture_perf_samples(contract, measure);
    let summary = measurement_capture::summarize_perf_samples(contract, &samples);
    profile_comparison::certify_against_baseline(contract, &summary);
    samples
}
