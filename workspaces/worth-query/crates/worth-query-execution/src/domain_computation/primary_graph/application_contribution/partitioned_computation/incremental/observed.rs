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

/// Captures existing execution counters before its work ceiling settles.
#[cfg(test)]
pub(in super::super::super) fn observe_full_tree<Tree, E>(
    cause: super::retained::WorthQueryPartitionedComputationFullCause,
    outcome: &Result<
        (Tree, ExecutionReport, worth_execution::ReductionMetrics),
        worth_execution::ReduceInputDenial<E>,
    >,
    mapped: &FullTreeMapWork,
) {
    if !mapped.observing {
        return;
    }
    use worth_execution::ReduceInputDenial;
    let metrics = match outcome {
        Ok((_, _, metrics)) => TreeMetrics::from_native(*metrics),
        Err(ReduceInputDenial::ReductionStopped { failure, .. }) => {
            TreeMetrics::from_native(failure.metrics)
        }
        Err(ReduceInputDenial::ScopeAdmission { report, .. }) => {
            let Some((work, _)) = report.accounted_cost() else {
                observe_tree(WorthQueryPartitionedTreeRun::Full(
                    cause,
                    TreeMetrics::from_native(worth_execution::ReductionMetrics::native()),
                ));
                return;
            };
            // Scope admission precedes kernels or follows a completed map and
            // shape. A completed tree settles inside its already held build bound.
            let visits = work
                .checked_sub(mapped.work())
                .expect("scope work covers every completed map kernel");
            if visits == 0 {
                assert_eq!(
                    mapped.work(),
                    0,
                    "a completed map must report its shape work"
                );
                return;
            }
            TreeMetrics::from_accounted(TreeAccounting {
                structural_visits: visits.into(),
                recombined_nodes: 0,
                combine_calls: 0,
                charged_work: visits.into(),
                charged_span: visits.into(),
            })
        }
        Err(ReduceInputDenial::MapStopped { .. }) => return,
    };
    observe_tree(WorthQueryPartitionedTreeRun::Full(cause, metrics));
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
                let visits = work
                    .checked_sub(map_work)
                    .expect("native scope covers settled map");
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

/// Captures the completed map's exposed kernel work for post-shape admission.
/// Scope admission after a completed map accepts every kernel's work.
#[cfg(test)]
pub(in super::super::super) struct FullTreeMapWork {
    observing: bool,
    work: std::sync::atomic::AtomicU64,
}

#[cfg(test)]
impl Default for FullTreeMapWork {
    fn default() -> Self {
        Self {
            observing: crate::domain_computation::execution_observation::is_installed(),
            work: Default::default(),
        }
    }
}

#[cfg(test)]
impl FullTreeMapWork {
    pub(in super::super::super) fn kernel<T, R, E>(
        &self,
        input: &T,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
        kernel: &impl Fn(
            &T,
            &mut worth_execution::MapKernelContext<'_, '_>,
        ) -> Result<R, worth_execution::MapKernelFailure<E>>,
    ) -> Result<R, worth_execution::MapKernelFailure<E>> {
        if !self.observing || !context.has_accounting() {
            return kernel(input, context);
        }
        let before = context.charged_work();
        let outcome = kernel(input, context);
        self.work.fetch_add(
            context
                .charged_work()
                .checked_sub(before)
                .expect("same kernel work counter is monotone"),
            std::sync::atomic::Ordering::Relaxed,
        );
        outcome
    }

    fn work(&self) -> u64 {
        self.work.load(std::sync::atomic::Ordering::Relaxed)
    }
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
