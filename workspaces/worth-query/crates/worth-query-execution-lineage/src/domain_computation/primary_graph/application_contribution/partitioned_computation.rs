//! The partitioned owner core types, incremental retention, and test observers.

pub use incremental::observe_retained_full_tree;
pub mod certification_reuse;
pub mod denial;
pub mod incremental;
pub mod items;
pub mod plan;
pub mod preparation_work;
pub mod routing;

pub use denial::{
    WorthQueryComputationInputDenial, WorthQueryComputationPartitionStop,
    WorthQueryComputationReadDenial, WorthQueryPartitionedComputationDenial,
    WorthQueryReductionInputDenial,
};
pub use incremental::WorthQueryPartitionedComputationFullCause;
pub use incremental::{
    discarded_computation_retention_on_this_thread_for_test,
    full_partitioned_computation_preparations_on_this_thread_for_test,
    partitioned_computation_runs_on_this_thread_for_test,
    partitioned_computation_tree_work_on_this_thread_for_test,
    published_partitioned_computations_on_this_thread_for_test,
    WorthQueryPartitionedComputationRunForTest, WorthQueryPartitionedTreeAccounting,
    WorthQueryPartitionedTreeMetrics, WorthQueryPartitionedTreeRebuildCause,
    WorthQueryPartitionedTreeRun, WorthQueryPublishedComputationStateForTest,
};
pub use incremental::{observe_discarded, observe_published};
pub use incremental::{
    Comparator, CompletedComputationRetention, ComputationDeposit, NativeComputationReads,
};
pub use incremental::{
    ComputationPrior, PriorAbsence, RetainedComputation, SealedComputationRetention,
    SealedComputationRun, Suppression,
};
pub use plan::{
    WorthQueryComputationPartitionMembers, WorthQueryComputationPartitionPlan,
    WorthQueryComputationPartitionView,
};

/// How partition results become one result.
///
/// The canonical tree fixes the association for a given set of partitions, so
/// a result that depends on it, such as a floating-point sum, is the same bits
/// on every run and at every worker count.
pub struct WorthQueryDeterministicReducer<Reduced> {
    pub identity: fn() -> Reduced,
    pub combine: fn(&Reduced, &Reduced) -> Reduced,
}

impl<Reduced> WorthQueryDeterministicReducer<Reduced> {
    /// Reduces over the canonical tree. `identity` is the result of no
    /// partitions.
    pub const fn canonical(
        identity: fn() -> Reduced,
        combine: fn(&Reduced, &Reduced) -> Reduced,
    ) -> Self {
        Self { identity, combine }
    }

    pub const fn identity(&self) -> fn() -> Reduced {
        self.identity
    }

    pub const fn combine(&self) -> fn(&Reduced, &Reduced) -> Reduced {
        self.combine
    }
}
