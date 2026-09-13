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
#[cfg(feature = "certification-support")]
mod scale_certification;
mod snapshot;
mod state;
mod topological_order;

pub(crate) use owner_lifecycle::{UiOverlayCompositionOwnerLifecycle, UiOverlayOwnerSources};

pub(crate) use dependency_index::{UiOverlayChangeSet, UiOverlayChangedBasis};
#[cfg(test)]
pub(crate) use extent::UiOverlayRegionExtent;
pub(crate) use extent::{
    UiOverlayMotionBinding, UiOverlayMotionSnapshot, UiOverlaySurfaceExtentSnapshot,
};
#[cfg(test)]
pub(crate) use planner::UiOverlayCompositionDenial;
pub(crate) use planner::{
    UiOverlayCapacityProfile, UiOverlayCompositionInput, UiOverlayCompositionState,
    UiOverlayPortalBinding, UiPreparedOverlayComposition,
};
#[cfg(test)]
pub(crate) use relation_graph::{
    UiCompiledOverlayRelationGraph, UiOverlayAnchor, UiOverlayRelationCompilationDenial,
    UiOverlayRelationKind,
};
pub(crate) use snapshot::{
    UiBackdropInstanceIdentity, UiOverlayApplicationGeneration, UiOverlayBackdropInstanceScope,
    UiOverlayBackdropRow, UiOverlayStackParticipant, UiOverlayStackSnapshot,
};
#[cfg(test)]
pub(crate) use snapshot::{UiOverlayExtent, UiOverlayParticipantIdentity};

#[cfg(feature = "certification-support")]
pub(crate) use scale_certification::overlay_scale_evidence;

#[cfg(test)]
#[path = "tests/mod.rs"]
mod tests;
