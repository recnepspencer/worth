//! Canonical reduction work, independent of the full-build contractual charge.

use worth_execution::{ReductionDenial, ReductionRunStop};

use super::super::WorthQueryPartitionedComputationDenial;
pub use worth_query_execution_contracts::facade::partitioned_tree_observation::{
    WorthQueryPartitionedTreeAccounting, WorthQueryPartitionedTreeMetrics,
    WorthQueryPartitionedTreeRebuildCause, WorthQueryPartitionedTreeRun,
};

/// Each path returns its outcome and work together; terminal construction consumes both.
pub struct ReportedTree<Tree, Stopped> {
    pub outcome: Result<Tree, WorthQueryPartitionedComputationDenial<Stopped>>,
    pub report: WorthQueryPartitionedTreeRun,
}

pub fn rebuild_cause<E>(
    reason: ReductionRunStop<E>,
) -> Result<WorthQueryPartitionedTreeRebuildCause, ReductionRunStop<E>> {
    use WorthQueryPartitionedTreeRebuildCause as Cause;
    match reason {
        cause @ (ReductionRunStop::Hook(_)
        | ReductionRunStop::Denial(
            ReductionDenial::Allocation(_)
            | ReductionDenial::Admission(_)
            | ReductionDenial::AccountingUnavailable,
        )) => Err(cause),
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

mod court_doors_for_test;
