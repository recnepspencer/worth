//! What the test observer is shown of each completed run.

use worth_foundational::facade::ExecutionReport;

use super::retained::WorthQueryPartitionedComputationRunForTest;
use super::tree_report::{
    WorthQueryPartitionedTreeAccounting as TreeAccounting,
    WorthQueryPartitionedTreeMetrics as TreeMetrics, WorthQueryPartitionedTreeRun,
};

thread_local! {
    static RUNS: std::cell::RefCell<Vec<(WorthQueryPartitionedComputationRunForTest, Option<ExecutionReport>)>> =
        const { std::cell::RefCell::new(Vec::new()) };
    static TREES: std::cell::RefCell<Vec<WorthQueryPartitionedTreeRun>> =
        const { std::cell::RefCell::new(Vec::new()) };
    static FULL_PREPARATIONS: std::cell::RefCell<Vec<super::retained::WorthQueryPartitionedComputationFullCause>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

/// Preparation chooses a cause before any owner call can stop the full run.
pub(in super::super) fn observe_full_preparation(
    cause: super::retained::WorthQueryPartitionedComputationFullCause,
) {
    if !crate::domain_computation::execution_observation::is_installed() {
        return;
    }
    if CLEANUP_REGISTERED.with(|registered| !registered.replace(true)) {
        crate::domain_computation::execution_observation::register_cleanup(clear_observations);
    }
    FULL_PREPARATIONS.with(|runs| runs.borrow_mut().push(cause));
}

/// Full preparations include failed runs, which have no completed-run report.
pub fn full_partitioned_computation_preparations_on_this_thread_for_test(
) -> Vec<super::retained::WorthQueryPartitionedComputationFullCause> {
    FULL_PREPARATIONS.with(|runs| std::mem::take(&mut *runs.borrow_mut()))
}

/// Shows a completed run to the test observer. The handler and the owner
/// never see how a run ran.
pub(super) fn observe(
    run: WorthQueryPartitionedComputationRunForTest,
    report: Option<ExecutionReport>,
) {
    if !crate::domain_computation::execution_observation::is_installed() {
        return;
    }
    if CLEANUP_REGISTERED.with(|registered| !registered.replace(true)) {
        crate::domain_computation::execution_observation::register_cleanup(clear_observations);
    }
    RUNS.with(|runs| runs.borrow_mut().push((run, report)));
}

/// Already-settled native evidence; observing work never interrogates the
/// kernel context or changes whether its actual result may be carried.
pub(in super::super::super) fn observe_retained_full_tree<R, F, E, Tree>(
    cause: super::retained::WorthQueryPartitionedComputationFullCause,
    outcome: &worth_execution::RetainedReduceOutcome<R, F, E, Tree>,
) {
    if !crate::domain_computation::execution_observation::is_installed() {
        return;
    }
    let metrics = outcome
        .reduction_metrics
        .map(TreeMetrics::from_native)
        .or_else(|| match &outcome.result {
            Err(worth_execution::RetainedReduceDenial::Reduction(
                worth_execution::ReduceInputDenial::ScopeAdmission { report, .. },
            )) => {
                let Some((work, _)) = report.accounted_cost() else {
                    return Some(TreeMetrics::from_native(
                        worth_execution::ReductionMetrics::native(),
                    ));
                };
                let map_work = outcome
                    .mapped_report
                    .and_then(|report| report.accounted_cost())
                    .map_or(0, |(work, _)| work);
                let visits = work.checked_sub(map_work)?;
                (visits != 0).then_some(TreeMetrics::from_accounted(TreeAccounting {
                    structural_visits: visits.into(),
                    recombined_nodes: 0,
                    combine_calls: 0,
                    charged_work: visits.into(),
                    charged_span: visits.into(),
                }))
            }
            _ => None,
        });
    if let Some(metrics) = metrics {
        observe_tree(WorthQueryPartitionedTreeRun::Full(cause, metrics));
    }
}

/// Records actual tree work on both completed and stopped executions.
pub(super) fn observe_tree(report: WorthQueryPartitionedTreeRun) {
    if !crate::domain_computation::execution_observation::is_installed() {
        return;
    }
    if CLEANUP_REGISTERED.with(|registered| !registered.replace(true)) {
        crate::domain_computation::execution_observation::register_cleanup(clear_observations);
    }
    TREES.with(|trees| trees.borrow_mut().push(report));
}

/// Takes terminal tree work, independently of partition execution causes.
pub fn partitioned_computation_tree_work_on_this_thread_for_test(
) -> Vec<WorthQueryPartitionedTreeRun> {
    TREES.with(|trees| std::mem::take(&mut *trees.borrow_mut()))
}

/// Takes every partitioned computation run completed on this thread since the
/// last call: how it ran, and execution's report of a full run.
pub fn partitioned_computation_runs_on_this_thread_for_test() -> Vec<(
    WorthQueryPartitionedComputationRunForTest,
    Option<ExecutionReport>,
)> {
    RUNS.with(|runs| std::mem::take(&mut *runs.borrow_mut()))
}

thread_local! { static CLEANUP_REGISTERED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) }; }
fn clear_observations() {
    CLEANUP_REGISTERED.with(|registered| registered.set(false));
    RUNS.with(|records| *records.borrow_mut() = Default::default());
    TREES.with(|records| *records.borrow_mut() = Default::default());
    FULL_PREPARATIONS.with(|records| *records.borrow_mut() = Default::default());
}

#[cfg(test)]
thread_local! { static RETAINED_FACT_READ: std::cell::Cell<usize> = const { std::cell::Cell::new(0) }; }
#[cfg(test)]
pub(super) struct RetainedFactComparison;
#[cfg(test)]
impl RetainedFactComparison {
    pub(super) fn enter() -> Self {
        RETAINED_FACT_READ.set(RETAINED_FACT_READ.get() + 1);
        Self
    }
}
#[cfg(test)]
impl Drop for RetainedFactComparison {
    fn drop(&mut self) {
        RETAINED_FACT_READ.set(RETAINED_FACT_READ.get() - 1);
    }
}
#[cfg(test)]
pub(crate) fn retained_fact_comparison_is_active_for_test() -> bool {
    crate::domain_computation::execution_observation::is_installed()
        && RETAINED_FACT_READ.get() != 0
}
