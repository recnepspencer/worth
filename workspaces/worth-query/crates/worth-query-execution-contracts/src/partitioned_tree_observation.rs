//! Pure observation types and thread-local recorders for partitioned computation runs.

use worth_execution::ReductionMetrics;
use worth_foundational::facade::ExecutionReport;

/// How one run of a partitioned computation ran.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryPartitionedComputationRunForTest {
    /// Only the partitions whose facts changed were gathered and computed;
    /// every other live partition was carried.
    Incremental {
        /// The partitions gathered and computed again, in ascending identity
        /// order.
        recomputed: Vec<worth_foundational::facade::PartitionIdentity>,
        /// Whether the membership call ran again because a fact it read
        /// moved. It does not claim that membership changed.
        membership_recomputed: bool,
    },
    /// Every partition was gathered and computed.
    Full(WorthQueryPartitionedComputationFullCause),
}

impl WorthQueryPartitionedComputationRunForTest {
    /// Why every partition ran, or `None` for an incremental run.
    pub const fn full_cause(&self) -> Option<WorthQueryPartitionedComputationFullCause> {
        match self {
            Self::Full(cause) => Some(*cause),
            Self::Incremental { .. } => None,
        }
    }
}

/// Why a run recomputed every partition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPartitionedComputationFullCause {
    FirstRun,
    Restored,
    Republished,
    NotProduced,
    Unmeasured,
    Moved,
    Evicted,
    InputChanged,
    OtherInstallation,
    OtherEdition,
    ObservationOverBudget,
    NoPriorHanded,
    NoProducerPrior,
    RetentionPolicy,
    SeveralComputations,
    CollisionSuppressed,
    Stopped,
    Unretained,
    IdentityCollision,
}

/// Why an incremental tree rebuilt its leaves instead of editing retained paths.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPartitionedTreeRebuildCause {
    WorkCeiling,
    EditMemory,
    EditCapacityOverflow,
    ReducerPanicked,
    /// A native edit refused; rebuilding decides the public outcome.
    EditDenied,
    ResultCapacityExceeded,
    WorkCounterOverflow,
}

/// Canonical serial-prefix counters; discarded speculative parallel work is excluded.
/// Wide sums preserve work even when an individual u64 attempt counter overflows.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryPartitionedTreeMetrics {
    accounting: Option<WorthQueryPartitionedTreeAccounting>,
}

/// Serial-prefix accounting exists only for explicitly accounted execution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryPartitionedTreeAccounting {
    pub structural_visits: u128,
    pub recombined_nodes: u128,
    pub combine_calls: u128,
    pub charged_work: u128,
    pub charged_span: u128,
}

impl WorthQueryPartitionedTreeMetrics {
    pub fn from_accounted(accounting: WorthQueryPartitionedTreeAccounting) -> Self {
        Self {
            accounting: Some(accounting),
        }
    }

    pub fn from_native(attempt: ReductionMetrics) -> Self {
        Self {
            accounting: attempt.accounted_metrics().map(|attempt| {
                let worth_execution::ReductionAccounting {
                    structural_visits,
                    recombined_nodes,
                    combine_calls,
                    charged_work,
                    charged_span,
                    ..
                } = attempt;
                WorthQueryPartitionedTreeAccounting {
                    structural_visits: structural_visits.into(),
                    recombined_nodes: recombined_nodes.into(),
                    combine_calls: combine_calls.into(),
                    charged_work: charged_work.into(),
                    charged_span: charged_span.into(),
                }
            }),
        }
    }

    pub const fn accounted_metrics(self) -> Option<WorthQueryPartitionedTreeAccounting> {
        self.accounting
    }

    pub fn followed_by(self, later: Self) -> Self {
        Self {
            accounting: self.accounting.zip(later.accounting).map(|(this, later)| {
                WorthQueryPartitionedTreeAccounting {
                    structural_visits: this.structural_visits + later.structural_visits,
                    recombined_nodes: this.recombined_nodes + later.recombined_nodes,
                    combine_calls: this.combine_calls + later.combine_calls,
                    charged_work: this.charged_work + later.charged_work,
                    charged_span: this.charged_span + later.charged_span,
                }
            }),
        }
    }
}

/// Full partition execution always builds; incremental execution edits or rebuilds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPartitionedTreeRun {
    Full(
        WorthQueryPartitionedComputationFullCause,
        WorthQueryPartitionedTreeMetrics,
    ),
    Edited(WorthQueryPartitionedTreeMetrics),
    Rebuilt(
        WorthQueryPartitionedTreeRebuildCause,
        WorthQueryPartitionedTreeMetrics,
    ),
}

impl WorthQueryPartitionedTreeRun {
    pub const fn metrics(self) -> WorthQueryPartitionedTreeMetrics {
        match self {
            Self::Full(_, metrics) | Self::Edited(metrics) | Self::Rebuilt(_, metrics) => metrics,
        }
    }

    pub const fn metrics_for_test(self) -> WorthQueryPartitionedTreeMetrics {
        self.metrics()
    }

    pub const fn full_cause_for_test(self) -> Option<WorthQueryPartitionedComputationFullCause> {
        match self {
            Self::Full(cause, _) => Some(cause),
            Self::Edited(_) | Self::Rebuilt(_, _) => None,
        }
    }
}

thread_local! {
    static RUNS: std::cell::RefCell<Vec<(WorthQueryPartitionedComputationRunForTest, Option<ExecutionReport>)>> =
        const { std::cell::RefCell::new(Vec::new()) };
    static TREES: std::cell::RefCell<Vec<WorthQueryPartitionedTreeRun>> =
        const { std::cell::RefCell::new(Vec::new()) };
    static FULL_PREPARATIONS: std::cell::RefCell<Vec<WorthQueryPartitionedComputationFullCause>> =
        const { std::cell::RefCell::new(Vec::new()) };
    static CLEANUP_REGISTERED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static RETAINED_FACT_READ: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

fn ensure_cleanup() {
    if CLEANUP_REGISTERED.with(|registered| !registered.replace(true)) {
        crate::execution_observation::register_cleanup(clear_observations);
    }
}

fn clear_observations() {
    CLEANUP_REGISTERED.with(|registered| registered.set(false));
    RUNS.with(|records| *records.borrow_mut() = Default::default());
    TREES.with(|records| *records.borrow_mut() = Default::default());
    FULL_PREPARATIONS.with(|records| *records.borrow_mut() = Default::default());
}

pub fn observe_full_preparation(cause: WorthQueryPartitionedComputationFullCause) {
    if !crate::execution_observation::is_installed() {
        return;
    }
    ensure_cleanup();
    FULL_PREPARATIONS.with(|runs| runs.borrow_mut().push(cause));
}

pub fn full_partitioned_computation_preparations_on_this_thread_for_test(
) -> Vec<WorthQueryPartitionedComputationFullCause> {
    FULL_PREPARATIONS.with(|runs| std::mem::take(&mut *runs.borrow_mut()))
}

pub fn observe(
    run: impl FnOnce() -> WorthQueryPartitionedComputationRunForTest,
    report: Option<ExecutionReport>,
) {
    if !crate::execution_observation::is_installed() {
        return;
    }
    let run = run();
    ensure_cleanup();
    RUNS.with(|runs| runs.borrow_mut().push((run, report)));
}

pub fn observe_retained_full_tree<R, F, E, Tree>(
    cause: WorthQueryPartitionedComputationFullCause,
    outcome: &worth_execution::RetainedReduceOutcome<R, F, E, Tree>,
) {
    if !crate::execution_observation::is_installed() {
        return;
    }
    let metrics = outcome
        .reduction_metrics
        .map(WorthQueryPartitionedTreeMetrics::from_native)
        .or_else(|| match &outcome.result {
            Err(worth_execution::RetainedReduceDenial::Reduction(
                worth_execution::ReduceInputDenial::ScopeAdmission { report, .. },
            )) => {
                let Some((work, _)) = report.accounted_cost() else {
                    return Some(WorthQueryPartitionedTreeMetrics::from_native(
                        worth_execution::ReductionMetrics::native(),
                    ));
                };
                let map_work = outcome
                    .mapped_report
                    .and_then(|report| report.accounted_cost())
                    .map_or(0, |(work, _)| work);
                let visits = work.checked_sub(map_work)?;
                (visits != 0).then_some(WorthQueryPartitionedTreeMetrics::from_accounted(
                    WorthQueryPartitionedTreeAccounting {
                        structural_visits: visits.into(),
                        recombined_nodes: 0,
                        combine_calls: 0,
                        charged_work: visits.into(),
                        charged_span: visits.into(),
                    },
                ))
            }
            _ => None,
        });
    if let Some(metrics) = metrics {
        observe_tree(WorthQueryPartitionedTreeRun::Full(cause, metrics));
    }
}

pub fn observe_tree(report: WorthQueryPartitionedTreeRun) {
    if !crate::execution_observation::is_installed() {
        return;
    }
    ensure_cleanup();
    TREES.with(|trees| trees.borrow_mut().push(report));
}

pub fn partitioned_computation_tree_work_on_this_thread_for_test(
) -> Vec<WorthQueryPartitionedTreeRun> {
    TREES.with(|trees| std::mem::take(&mut *trees.borrow_mut()))
}

pub fn partitioned_computation_runs_on_this_thread_for_test() -> Vec<(
    WorthQueryPartitionedComputationRunForTest,
    Option<ExecutionReport>,
)> {
    RUNS.with(|runs| std::mem::take(&mut *runs.borrow_mut()))
}

pub struct RetainedFactComparison;
impl RetainedFactComparison {
    pub fn enter() -> Self {
        RETAINED_FACT_READ.set(RETAINED_FACT_READ.get() + 1);
        Self
    }
}
impl Drop for RetainedFactComparison {
    fn drop(&mut self) {
        RETAINED_FACT_READ.set(RETAINED_FACT_READ.get() - 1);
    }
}

pub fn retained_fact_comparison_is_active_for_test() -> bool {
    crate::execution_observation::is_installed() && RETAINED_FACT_READ.get() != 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution_observation::ExecutionObservationGuard;

    #[test]
    fn an_unobserved_run_builds_no_payload() {
        assert!(!crate::execution_observation::is_installed());
        observe(|| panic!("payload built without an observer"), None);
        let observation = ExecutionObservationGuard::install_on_this_thread();
        let mut built = 0;
        observe(
            || {
                built += 1;
                WorthQueryPartitionedComputationRunForTest::Incremental {
                    recomputed: Vec::new(),
                    membership_recomputed: false,
                }
            },
            None,
        );
        assert_eq!(built, 1);
        assert_eq!(
            partitioned_computation_runs_on_this_thread_for_test().len(),
            1
        );
        drop(observation);
    }
}
