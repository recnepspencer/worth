pub(crate) mod effect;
pub(crate) mod execution;
pub(crate) mod graph;
pub(crate) mod observer;
pub(crate) mod scratch;
pub(crate) mod strategy;

pub(crate) use effect::{ApplyCommitPacket, PreparedParallelApplyCommitPacket};
pub(crate) use graph::{BranchMutationRecord, BranchStructuralDelta};
pub use graph::{
    SignalGraph, SignalGraphLifecycleProbe, SignalGraphReconstitution,
    SignalGraphReconstitutionReport,
};
pub use observer::{GraphMaterializer, GraphObserver};
pub use scratch::ScratchLeaseKind;
pub(crate) use scratch::TraversalScratch;
pub use strategy::{EvaluationStrategy, GcPressure, ObservationLevel};
