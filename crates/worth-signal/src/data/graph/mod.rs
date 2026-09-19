mod compaction;
mod construction;
mod diagnostics_access;
mod lifecycle;
mod runtime;
pub(crate) mod storage;
mod topology;

pub(crate) use construction::node_builder;
pub use construction::NodeBuilder;
pub(crate) use runtime::effect::{
    DirectInvalidationPreparationReceipt, OutputCommitPublicationReceipt,
};
pub(crate) use runtime::graph as signal_graph;
pub(crate) use runtime::scratch;
pub use runtime::ScratchLeaseKind;
pub(crate) use runtime::TraversalScratch;
#[cfg_attr(not(feature = "parallel"), allow(unused_imports))]
pub(crate) use runtime::{ApplyCommitPacket, PreparedParallelApplyCommitPacket};
#[allow(unused_imports)]
pub(crate) use runtime::{BranchMutationRecord, BranchStructuralDelta};
pub use runtime::{
    EvaluationStrategy, GcPressure, GraphMaterializer, GraphObserver, ObservationLevel,
    ParallelismHint,
};
pub use runtime::{
    SignalGraph, SignalGraphLifecycleProbe, SignalGraphReconstitution,
    SignalGraphReconstitutionReport,
};
#[cfg(test)]
pub(crate) use storage::checked_segment_component_for_test;
pub(crate) use storage::{
    DependencyEdgeStore, DependencySetId, SubscriberEdgeStore, SubscriberSetId,
};
pub(crate) use topology::ReverseSubscriptionIndex;
pub(crate) use topology::{
    PendingRevalidationNodeProjection, PendingRevalidationPreparationDenial,
    PreparedPendingRevalidationIndex, PreparedPendingRevalidationResolution,
    PreparedRetainedPendingRevalidationIndex,
};

pub(crate) use topology::subscription_candidates;

pub(crate) use topology::waiter_preparation_work;
