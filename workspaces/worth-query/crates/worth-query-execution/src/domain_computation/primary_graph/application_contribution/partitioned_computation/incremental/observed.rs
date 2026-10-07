//! What the test observer is shown of each completed run.

use worth_foundational::facade::ExecutionReport;

use super::retained::WorthQueryPartitionedComputationRun;
use super::tree_report::WorthQueryPartitionedTreeRun;

#[cfg(any(test, feature = "test-query-execution-observer"))]
thread_local! {
    static RUNS: std::cell::RefCell<Vec<(WorthQueryPartitionedComputationRun, Option<ExecutionReport>)>> =
        const { std::cell::RefCell::new(Vec::new()) };
    static TREES: std::cell::RefCell<Vec<(WorthQueryPartitionedComputationRun, WorthQueryPartitionedTreeRun)>> =
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

/// Captures existing execution counters before its work ceiling settles.
pub(in super::super::super) fn observe_full_tree<Tree, E>(
    cause: super::retained::WorthQueryPartitionedComputationFullCause,
    outcome: &Result<
        (Tree, ExecutionReport, worth_execution::ReductionMetrics),
        worth_execution::ReduceInputDenial<E>,
    >,
) {
    use worth_execution::ReduceInputDenial;
    let metrics = match outcome {
        Ok((_, _, metrics)) => *metrics,
        Err(ReduceInputDenial::ReductionStopped { failure, .. }) => failure.metrics,
        Err(ReduceInputDenial::ScopeAdmission { .. } | ReduceInputDenial::MapStopped { .. }) => {
            return
        }
    };
    observe_tree(
        WorthQueryPartitionedComputationRun::Full(cause),
        WorthQueryPartitionedTreeRun::full_attempt(metrics),
    );
}

/// Records actual tree work on both completed and stopped executions.
pub(super) fn observe_tree(
    partitions: WorthQueryPartitionedComputationRun,
    report: WorthQueryPartitionedTreeRun,
) {
    #[cfg(any(test, feature = "test-query-execution-observer"))]
    TREES.with(|trees| trees.borrow_mut().push((partitions, report)));
    #[cfg(not(any(test, feature = "test-query-execution-observer")))]
    let _ = (partitions, report);
}

/// Takes terminal tree work, independently of partition execution causes.
#[cfg(any(test, feature = "test-query-execution-observer"))]
pub fn partitioned_computation_tree_work_on_this_thread_for_test() -> Vec<(
    WorthQueryPartitionedComputationRun,
    WorthQueryPartitionedTreeRun,
)> {
    TREES.with(|trees| std::mem::take(&mut *trees.borrow_mut()))
}

/// Takes every partitioned computation run completed on this thread since the
/// last call: how it ran, and execution's report of a full run.
#[cfg(any(test, feature = "test-query-execution-observer"))]
pub fn partitioned_computation_runs_on_this_thread_for_test(
) -> Vec<(WorthQueryPartitionedComputationRun, Option<ExecutionReport>)> {
    RUNS.with(|runs| std::mem::take(&mut *runs.borrow_mut()))
}
