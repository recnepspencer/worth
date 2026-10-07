//! What the test observer is shown of each completed run.

use worth_foundational::facade::ExecutionReport;

use super::retained::WorthQueryPartitionedComputationRun;

#[cfg(any(test, feature = "test-query-execution-observer"))]
thread_local! {
    static RUNS: std::cell::RefCell<Vec<(WorthQueryPartitionedComputationRun, Option<ExecutionReport>)>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

/// Shows a completed run to the test observer. The handler and the owner
/// never see how a run ran.
pub(super) fn observe(run: WorthQueryPartitionedComputationRun, report: Option<ExecutionReport>) {
    #[cfg(any(test, feature = "test-query-execution-observer"))]
    RUNS.with(|runs| runs.borrow_mut().push((run, report)));
    #[cfg(not(any(test, feature = "test-query-execution-observer")))]
    let _ = (run, report);
}

/// Takes every partitioned computation run completed on this thread since the
/// last call: how it ran, and execution's report of a full run.
#[cfg(any(test, feature = "test-query-execution-observer"))]
pub fn partitioned_computation_runs_on_this_thread_for_test(
) -> Vec<(WorthQueryPartitionedComputationRun, Option<ExecutionReport>)> {
    RUNS.with(|runs| std::mem::take(&mut *runs.borrow_mut()))
}
