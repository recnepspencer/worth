//! What the test observer is shown of each completed run.

use worth_foundational::facade::ExecutionReport;

use super::retained::WorthQueryPartitionedComputationRunForTest;
use super::tree_report::WorthQueryPartitionedTreeRun;

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
        Ok((_, _, metrics)) => *metrics,
        Err(ReduceInputDenial::ReductionStopped { failure, .. }) => failure.metrics,
        Err(ReduceInputDenial::ScopeAdmission { report, .. }) => {
            // Scope admission precedes kernels or follows a completed map and
            // shape. A completed tree settles inside its already held build bound.
            let visits = report
                .charged_work()
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
            worth_execution::ReductionMetrics {
                structural_visits: visits,
                recombined_nodes: 0,
                combine_calls: 0,
                charged_work: visits,
                charged_span: visits,
            }
        }
        Err(ReduceInputDenial::MapStopped { .. }) => return,
    };
    observe_tree(WorthQueryPartitionedTreeRun::Full(
        cause,
        super::tree_report::WorthQueryPartitionedTreeMetrics::from_native(metrics),
    ));
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
pub(in super::super::super) struct FullTreeMapWork {
    observing: bool,
    work: std::sync::atomic::AtomicU64,
}

impl Default for FullTreeMapWork {
    fn default() -> Self {
        Self {
            observing: crate::domain_computation::execution_observation::is_installed(),
            work: Default::default(),
        }
    }
}

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
        if !self.observing {
            return kernel(input, context);
        }
        let before = context.remaining_work();
        let outcome = kernel(input, context);
        self.work.fetch_add(
            before - context.remaining_work(),
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
