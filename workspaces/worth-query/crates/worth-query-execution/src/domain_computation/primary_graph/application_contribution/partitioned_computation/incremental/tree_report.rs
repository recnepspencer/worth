//! Canonical reduction work, independent of the full-build contractual charge.

use worth_execution::{ReductionDenial, ReductionMetrics, ReductionRunStop};

use super::super::WorthQueryPartitionedComputationDenial;
use super::retained::WorthQueryPartitionedComputationFullCause;
#[cfg(any(test, feature = "test-query-execution-observer"))]
use super::retained::WorthQueryPartitionedComputationRun;

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
    pub structural_visits: u128,
    pub recombined_nodes: u128,
    pub combine_calls: u128,
    pub charged_work: u128,
    pub charged_span: u128,
}

impl WorthQueryPartitionedTreeMetrics {
    pub(super) fn from_native(attempt: ReductionMetrics) -> Self {
        let ReductionMetrics {
            structural_visits,
            recombined_nodes,
            combine_calls,
            charged_work,
            charged_span,
        } = attempt;
        Self {
            structural_visits: structural_visits.into(),
            recombined_nodes: recombined_nodes.into(),
            combine_calls: combine_calls.into(),
            charged_work: charged_work.into(),
            charged_span: charged_span.into(),
        }
    }
}

impl WorthQueryPartitionedTreeMetrics {
    pub(super) fn followed_by(self, later: Self) -> Self {
        // A usize-sized edit sequence plus one rebuild of u64 counters fits u128.
        Self {
            structural_visits: self.structural_visits + later.structural_visits,
            recombined_nodes: self.recombined_nodes + later.recombined_nodes,
            combine_calls: self.combine_calls + later.combine_calls,
            charged_work: self.charged_work + later.charged_work,
            charged_span: self.charged_span + later.charged_span,
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
    #[cfg(any(test, feature = "test-query-execution-observer"))]
    pub const fn metrics(self) -> WorthQueryPartitionedTreeMetrics {
        match self {
            Self::Full(_, metrics) | Self::Edited(metrics) | Self::Rebuilt(_, metrics) => metrics,
        }
    }

    #[cfg(any(test, feature = "test-query-execution-observer"))]
    pub const fn partitions(self) -> WorthQueryPartitionedComputationRun {
        match self {
            Self::Full(cause, _) => WorthQueryPartitionedComputationRun::Full(cause),
            Self::Edited(_) | Self::Rebuilt(_, _) => {
                WorthQueryPartitionedComputationRun::Incremental
            }
        }
    }
}

/// Each path returns its outcome and work together; terminal construction consumes both.
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
        ReductionRunStop::Panic | ReductionRunStop::Denial(ReductionDenial::ReducerPanic) => {
            Ok(Cause::ReducerPanicked)
        }
        ReductionRunStop::ResultCapacityExceeded
        | ReductionRunStop::Denial(ReductionDenial::ResultCapacityExceeded) => {
            Ok(Cause::ResultCapacityExceeded)
        }
        ReductionRunStop::WorkCounterOverflow
        | ReductionRunStop::Denial(ReductionDenial::WorkCounterOverflow) => {
            Ok(Cause::WorkCounterOverflow)
        }
        ReductionRunStop::Denial(
            ReductionDenial::InvalidCanonicalEncoding
            | ReductionDenial::IdentitiesNotCanonical
            | ReductionDenial::ValueCountMismatch
            | ReductionDenial::CoverageMismatch
            | ReductionDenial::UnknownIdentity(_)
            | ReductionDenial::IdentityAlreadyPresent(_),
        ) => Ok(Cause::EditDenied),
    }
}
