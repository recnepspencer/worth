//! What the test observer is shown of each completed run.

use worth_foundational::facade::ExecutionReport;

use super::retained::WorthQueryPartitionedComputationRun;
use super::tree_report::WorthQueryPartitionedTreeRun;

#[cfg(any(test, feature = "test-query-execution-observer"))]
thread_local! {
    static RUNS: std::cell::RefCell<Vec<(WorthQueryPartitionedComputationRun, Option<ExecutionReport>)>> =
        const { std::cell::RefCell::new(Vec::new()) };
    static TREES: std::cell::RefCell<Vec<WorthQueryPartitionedTreeRun>> =
        const { std::cell::RefCell::new(Vec::new()) };
    #[cfg(feature = "test-query-execution-observer")]
    static FULL_PREPARATIONS: std::cell::RefCell<Vec<super::retained::WorthQueryPartitionedComputationFullCause>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

/// Preparation chooses a cause before any owner call can stop the full run.
pub(in super::super) fn observe_full_preparation(
    cause: super::retained::WorthQueryPartitionedComputationFullCause,
) {
    #[cfg(feature = "test-query-execution-observer")]
    FULL_PREPARATIONS.with(|runs| runs.borrow_mut().push(cause));
    #[cfg(not(feature = "test-query-execution-observer"))]
    let _ = cause;
}

/// Full preparations include failed runs, which have no completed-run report.
#[cfg(feature = "test-query-execution-observer")]
pub fn full_partitioned_computation_preparations_on_this_thread_for_test(
) -> Vec<super::retained::WorthQueryPartitionedComputationFullCause> {
    FULL_PREPARATIONS.with(|runs| std::mem::take(&mut *runs.borrow_mut()))
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
    mapped: &FullTreeMapWork,
) {
    use worth_execution::ReduceInputDenial;
    #[cfg(not(any(test, feature = "test-query-execution-observer")))]
    let _ = mapped;
    let metrics = match outcome {
        Ok((_, _, metrics)) => *metrics,
        Err(ReduceInputDenial::ReductionStopped { failure, .. }) => failure.metrics,
        #[cfg(any(test, feature = "test-query-execution-observer"))]
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
        #[cfg(not(any(test, feature = "test-query-execution-observer")))]
        Err(ReduceInputDenial::ScopeAdmission { .. }) => return,
        Err(ReduceInputDenial::MapStopped { .. }) => return,
    };
    observe_tree(WorthQueryPartitionedTreeRun::Full(
        cause,
        super::tree_report::WorthQueryPartitionedTreeMetrics::from_native(metrics),
    ));
}

/// Records actual tree work on both completed and stopped executions.
pub(super) fn observe_tree(report: WorthQueryPartitionedTreeRun) {
    #[cfg(any(test, feature = "test-query-execution-observer"))]
    TREES.with(|trees| trees.borrow_mut().push(report));
    #[cfg(not(any(test, feature = "test-query-execution-observer")))]
    let _ = report;
}

/// Takes terminal tree work, independently of partition execution causes.
#[cfg(any(test, feature = "test-query-execution-observer"))]
pub fn partitioned_computation_tree_work_on_this_thread_for_test(
) -> Vec<WorthQueryPartitionedTreeRun> {
    TREES.with(|trees| std::mem::take(&mut *trees.borrow_mut()))
}

/// Takes every partitioned computation run completed on this thread since the
/// last call: how it ran, and execution's report of a full run.
#[cfg(any(test, feature = "test-query-execution-observer"))]
pub fn partitioned_computation_runs_on_this_thread_for_test(
) -> Vec<(WorthQueryPartitionedComputationRun, Option<ExecutionReport>)> {
    RUNS.with(|runs| std::mem::take(&mut *runs.borrow_mut()))
}

/// Captures the completed map's exposed kernel work for post-shape admission.
/// Scope admission after a completed map accepts every kernel's work.
#[derive(Default)]
pub(in super::super::super) struct FullTreeMapWork {
    #[cfg(any(test, feature = "test-query-execution-observer"))]
    work: std::sync::atomic::AtomicU64,
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
        #[cfg(any(test, feature = "test-query-execution-observer"))]
        let before = context.remaining_work();
        let outcome = kernel(input, context);
        #[cfg(any(test, feature = "test-query-execution-observer"))]
        self.work.fetch_add(
            before - context.remaining_work(),
            std::sync::atomic::Ordering::Relaxed,
        );
        outcome
    }

    #[cfg(any(test, feature = "test-query-execution-observer"))]
    fn work(&self) -> u64 {
        self.work.load(std::sync::atomic::Ordering::Relaxed)
    }
}
