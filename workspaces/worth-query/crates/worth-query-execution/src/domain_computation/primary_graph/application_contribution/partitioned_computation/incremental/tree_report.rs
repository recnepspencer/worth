//! Actual reduction work, independent of the full-build contractual charge.

use worth_execution::{ReductionDenial, ReductionMetrics, ReductionRunStop};

use super::super::WorthQueryPartitionedComputationDenial;

/// Why a tree rebuilt its leaves instead of editing retained paths.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPartitionedTreeRebuildCause {
    FullBuild,
    WorkCeiling,
    EditMemory,
    EditCapacityOverflow,
    ReducerPanicked,
    EditDenied(ReductionDenial),
    ResultCapacityExceeded,
    WorkCounterOverflow,
}

/// Every attempt's actual counters. Wide sums preserve work even when an
/// individual execution attempt stops because its u64 work counter overflowed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryPartitionedTreeMetrics {
    pub structural_visits: u128,
    pub recombined_nodes: u128,
    pub combine_calls: u128,
    pub charged_work: u128,
    pub charged_span: u128,
}

impl WorthQueryPartitionedTreeMetrics {
    pub(super) const fn before_attempts() -> Self {
        Self {
            structural_visits: 0,
            recombined_nodes: 0,
            combine_calls: 0,
            charged_work: 0,
            charged_span: 0,
        }
    }

    pub(super) fn include(&mut self, attempt: ReductionMetrics) {
        // A usize-sized edit sequence plus one rebuild, each with u64
        // counters, fits in u128 on supported platforms. This never charges work.
        let ReductionMetrics {
            structural_visits,
            recombined_nodes,
            combine_calls,
            charged_work,
            charged_span,
        } = attempt;
        self.structural_visits += u128::from(structural_visits);
        self.recombined_nodes += u128::from(recombined_nodes);
        self.combine_calls += u128::from(combine_calls);
        self.charged_work += u128::from(charged_work);
        self.charged_span += u128::from(charged_span);
    }
}

/// The tree execution dimension, including work before a terminal stop and
/// before a rebuild. Partition execution has its own independent cause.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPartitionedTreeRun {
    Edited(WorthQueryPartitionedTreeMetrics),
    Rebuilt(
        WorthQueryPartitionedTreeRebuildCause,
        WorthQueryPartitionedTreeMetrics,
    ),
}

impl WorthQueryPartitionedTreeRun {
    #[cfg(any(test, feature = "test-query-execution-observer"))]
    pub const fn metrics(self) -> WorthQueryPartitionedTreeMetrics {
        match self {
            Self::Edited(metrics) | Self::Rebuilt(_, metrics) => metrics,
        }
    }

    pub(super) fn full_attempt(metrics: ReductionMetrics) -> Self {
        let mut actual = WorthQueryPartitionedTreeMetrics::before_attempts();
        actual.include(metrics);
        Self::Rebuilt(WorthQueryPartitionedTreeRebuildCause::FullBuild, actual)
    }
}

/// All terminal tree-update paths construct both the outcome and its work.
/// There is no default or constructor that can omit the report.
pub(super) struct ReportedTree<Tree, Stopped> {
    pub(super) outcome: Result<Tree, WorthQueryPartitionedComputationDenial<Stopped>>,
    pub(super) report: WorthQueryPartitionedTreeRun,
}

pub(super) fn rebuild_cause<E>(
    reason: ReductionRunStop<E>,
) -> Result<WorthQueryPartitionedTreeRebuildCause, E> {
    use WorthQueryPartitionedTreeRebuildCause as Cause;
    match reason {
        ReductionRunStop::Hook(stop) => Err(stop),
        ReductionRunStop::Panic => Ok(Cause::ReducerPanicked),
        ReductionRunStop::Denial(denial) => Ok(Cause::EditDenied(denial)),
        ReductionRunStop::ResultCapacityExceeded => Ok(Cause::ResultCapacityExceeded),
        ReductionRunStop::WorkCounterOverflow => Ok(Cause::WorkCounterOverflow),
    }
}
