#![allow(
    dead_code,
    reason = "Gate 1 retains the overlay composition authority for later publication"
)]

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

#[allow(
    unused_imports,
    reason = "Gate 1 retains the sealed overlay binding export for later portal publication"
)]
pub(super) use binding_export::UiOverlayPortalBindingExport;
#[allow(
    unused_imports,
    reason = "Gate 1 retains the sealed overlay motion export for later motion publication"
)]
pub(super) use motion_export::UiOverlayMotionOwnerExport;
#[allow(
    unused_imports,
    reason = "Gate 1 retains the overlay composition owner for later lifecycle wiring"
)]
pub(super) use owner::UiOverlayCompositionOwner;
#[allow(
    unused_imports,
    reason = "Gate 1 retains the typed overlay owner input seam for later admission"
)]
pub(super) use owner_input::{UiOverlayOwnerExportDenial, UiOverlayOwnerExportVector};
#[allow(
    unused_imports,
    reason = "Gate 1 retains the typed overlay owner lifecycle seam for later publication"
)]
pub(crate) use owner_lifecycle::{
    UiOverlayCompositionOwnerLifecycle, UiOverlayOwnerBridgeDenial, UiOverlayOwnerSources,
};
#[allow(
    unused_imports,
    reason = "Gate 1 retains the sealed Portal-to-overlay owner export for later composition"
)]
pub(super) use portal_export::UiOverlayPortalOwnerExport;

#[allow(
    unused_imports,
    reason = "Gate 1 retains overlay dependency facts for later invalidation consumers"
)]
pub(crate) use dependency_index::{
    UiOverlayChangeSet, UiOverlayChangedBasis, UiOverlayDependencyIndex, UiOverlayDependencyKind,
};
#[allow(
    unused_imports,
    reason = "Gate 1 retains overlay extent and motion facts for later composition consumers"
)]
pub(crate) use extent::{
    UiOverlayMotionBinding, UiOverlayMotionSnapshot, UiOverlayRegionExtent,
    UiOverlaySurfaceExtentSnapshot,
};
#[allow(
    unused_imports,
    reason = "Gate 1 retains overlay planning types for later publication"
)]
pub(crate) use planner::{
    UiOverlayCapacityProfile, UiOverlayCommitDenial, UiOverlayCompositionDenial,
    UiOverlayCompositionInput, UiOverlayCompositionState, UiOverlayPortalBinding,
    UiPreparedOverlayComposition,
};
#[allow(
    unused_imports,
    reason = "Gate 1 retains overlay relation graph types for later ordering consumers"
)]
pub(crate) use relation_graph::{
    UiCompiledOverlayRelationGraph, UiOverlayAnchor, UiOverlayRelationCompilationDenial,
    UiOverlayRelationKind,
};
#[allow(
    unused_imports,
    reason = "Gate 1 retains overlay snapshot facts for later mounting consumers"
)]
pub(crate) use snapshot::{
    UiBackdropInstanceIdentity, UiOverlayApplicationGeneration, UiOverlayBackdropInstanceScope,
    UiOverlayBackdropRow, UiOverlayExtent, UiOverlayParticipantIdentity, UiOverlayStackParticipant,
    UiOverlayStackSnapshot,
};

#[cfg(test)]
#[path = "tests/mod.rs"]
mod tests;
