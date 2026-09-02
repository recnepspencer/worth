mod dependency_index;
mod extent;
mod full_rebuild;
mod materialization;
mod order;
mod planner;
mod relation_cache;
mod relation_graph;
mod snapshot;
mod state;
mod topological_order;

pub(crate) use dependency_index::{
    UiOverlayChangeSet, UiOverlayChangedBasis, UiOverlayDependencyIndex, UiOverlayDependencyKind,
};
pub(crate) use extent::{
    UiOverlayMotionBinding, UiOverlayMotionSnapshot, UiOverlayRegionExtent,
    UiOverlaySurfaceExtentSnapshot,
};
pub(crate) use planner::{
    UiOverlayCapacityProfile, UiOverlayCommitDenial, UiOverlayCompositionDenial,
    UiOverlayCompositionInput, UiOverlayCompositionState, UiOverlayPortalBinding,
    UiPreparedOverlayComposition,
};
pub(crate) use relation_graph::{
    UiCompiledOverlayRelationGraph, UiOverlayAnchor, UiOverlayRelationCompilationDenial,
    UiOverlayRelationKind,
};
pub(crate) use snapshot::{
    UiBackdropInstanceIdentity, UiOverlayApplicationGeneration, UiOverlayBackdropInstanceScope,
    UiOverlayBackdropRow, UiOverlayParticipantIdentity, UiOverlayStackParticipant,
    UiOverlayStackSnapshot,
};

#[cfg(test)]
#[path = "tests/mod.rs"]
mod tests;
