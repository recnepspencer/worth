mod binding_export;
mod coordinator;
mod dependency_index;
mod extent;
mod full_rebuild;
mod materialization;
mod motion_export;
mod order;
mod owner;
mod owner_bridge;
mod owner_input;
mod owner_lifecycle;
mod planner;
mod portal_export;
mod relation_cache;
mod relation_graph;
mod snapshot;
mod state;
mod topological_order;

pub(super) use binding_export::UiOverlayPortalBindingExport;
pub(super) use motion_export::UiOverlayMotionOwnerExport;
pub(super) use owner::UiOverlayCompositionOwner;
pub(super) use owner_input::{UiOverlayOwnerExportDenial, UiOverlayOwnerExportVector};
pub(super) use owner_lifecycle::{
    UiOverlayCompositionOwnerLifecycle, UiOverlayOwnerBridgeDenial, UiOverlayOwnerSources,
};
pub(super) use portal_export::UiOverlayPortalOwnerExport;

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
    UiOverlayBackdropRow, UiOverlayExtent, UiOverlayParticipantIdentity, UiOverlayStackParticipant,
    UiOverlayStackSnapshot,
};

#[cfg(test)]
#[path = "tests/mod.rs"]
mod tests;
