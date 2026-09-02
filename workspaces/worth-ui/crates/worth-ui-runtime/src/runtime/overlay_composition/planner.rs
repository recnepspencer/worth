use worth_ui_dsl::{
    UiBackdropDeclaration, UiMosaicRegionDeclarationIdentity, UiPortalDeclarationId,
};

use super::dependency_index::UiOverlayDependencyIndex;
use super::extent::{UiOverlayMotionSnapshot, UiOverlaySurfaceExtentSnapshot};
use super::relation_cache::UiOverlayRelationCache;
use super::snapshot::{UiOverlayApplicationGeneration, UiOverlayStackSnapshot};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct UiOverlayPortalBinding {
    declaration: UiPortalDeclarationId,
    portal: crate::runtime::portal::UiPortalIdentity,
}

impl UiOverlayPortalBinding {
    #[cfg(test)]
    pub(in crate::runtime::overlay_composition) const fn new(
        declaration: UiPortalDeclarationId,
        portal: crate::runtime::portal::UiPortalIdentity,
    ) -> Self {
        Self {
            declaration,
            portal,
        }
    }

    pub(crate) const fn declaration(self) -> UiPortalDeclarationId {
        self.declaration
    }

    pub(crate) const fn portal(self) -> crate::runtime::portal::UiPortalIdentity {
        self.portal
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiOverlayCapacityProfile {
    pub(super) max_backdrop_declarations: usize,
    pub(super) max_portal_rows: usize,
    pub(super) max_backdrop_rows: usize,
    pub(super) max_order_rows: usize,
    pub(super) max_relation_edges: usize,
}

impl UiOverlayCapacityProfile {
    #[cfg(test)]
    pub(crate) const fn qualified() -> Self {
        Self {
            max_backdrop_declarations: worth_ui_dsl::UI_APPEARANCE_BACKDROP_RELATION_CAPACITY,
            max_portal_rows: 1_024,
            max_backdrop_rows: 1_024,
            max_order_rows: 2_048,
            max_relation_edges: worth_ui_dsl::UI_APPEARANCE_BACKDROP_RELATION_CAPACITY * 2,
        }
    }
}

#[cfg(test)]
impl Default for UiOverlayCapacityProfile {
    fn default() -> Self {
        Self::qualified()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiOverlayReservation {
    pub(crate) portal_rows: usize,
    pub(crate) backdrop_rows: usize,
    pub(crate) order_rows: usize,
    pub(crate) relation_edges: usize,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct UiOverlayPlanCounters {
    pub(super) portal_stack_rows_read: usize,
    pub(super) portal_binding_entries_read: usize,
    pub(super) backdrop_declarations_selected: usize,
    pub(super) overlay_relation_edges_visited: usize,
    pub(super) backdrop_mechanics_changed: usize,
    pub(super) backdrop_commands_replayed: usize,
    pub(super) unrelated_neighborhoods_touched: usize,
}

impl UiOverlayPlanCounters {
    pub(crate) const fn portal_stack_rows_read(self) -> usize {
        self.portal_stack_rows_read
    }
    pub(crate) const fn portal_binding_entries_read(self) -> usize {
        self.portal_binding_entries_read
    }
    pub(crate) const fn backdrop_declarations_selected(self) -> usize {
        self.backdrop_declarations_selected
    }
    pub(crate) const fn overlay_relation_edges_visited(self) -> usize {
        self.overlay_relation_edges_visited
    }
    pub(crate) const fn backdrop_mechanics_changed(self) -> usize {
        self.backdrop_mechanics_changed
    }
    pub(crate) const fn backdrop_commands_replayed(self) -> usize {
        self.backdrop_commands_replayed
    }
    pub(crate) const fn unrelated_neighborhoods_touched(self) -> usize {
        self.unrelated_neighborhoods_touched
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum UiOverlayCompositionDenial {
    InvalidDeclarationRevision,
    BackdropDeclarationCapacityExceeded {
        observed: usize,
        maximum: usize,
    },
    DuplicateBackdropIdentity,
    Relation(super::relation_graph::UiOverlayRelationCompilationDenial),
    DuplicatePortalBinding,
    DuplicatePortalSnapshotRow(crate::runtime::portal::UiPortalIdentity),
    ForeignPortalBinding {
        portal: crate::runtime::portal::UiPortalIdentity,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    },
    MissingPortalDeclarationBinding(crate::runtime::portal::UiPortalIdentity),
    MissingPortalSnapshotRow(crate::runtime::portal::UiPortalIdentity),
    PortalRowCapacityExceeded {
        observed: usize,
        maximum: usize,
    },
    BackdropRowCapacityExceeded {
        observed: usize,
        maximum: usize,
    },
    OverlayOrderCapacityExceeded {
        observed: usize,
        maximum: usize,
    },
    RelationEdgeCapacityExceeded {
        observed: usize,
        maximum: usize,
    },
    MissingPortalAnchor {
        backdrop: worth_ui_dsl::UiBackdropIdentity,
        portal: UiPortalDeclarationId,
    },
    AmbiguousPortalAnchor {
        backdrop: worth_ui_dsl::UiBackdropIdentity,
        portal: UiPortalDeclarationId,
    },
    MissingBackdropAnchor {
        backdrop: worth_ui_dsl::UiBackdropIdentity,
        anchor: worth_ui_dsl::UiBackdropIdentity,
    },
    AmbiguousBackdropAnchor {
        backdrop: worth_ui_dsl::UiBackdropIdentity,
        anchor: worth_ui_dsl::UiBackdropIdentity,
    },
    CrossScopeBackdropAnchor {
        backdrop: worth_ui_dsl::UiBackdropIdentity,
        anchor: worth_ui_dsl::UiBackdropIdentity,
    },
    ForeignSurfaceExtent {
        backdrop: worth_ui_dsl::UiBackdropIdentity,
    },
    MissingRegionExtent {
        backdrop: worth_ui_dsl::UiBackdropIdentity,
        region: UiMosaicRegionDeclarationIdentity,
    },
    MissingMotionBasis {
        backdrop: worth_ui_dsl::UiBackdropIdentity,
        portal: UiPortalDeclarationId,
    },
    MissingMotionBinding {
        backdrop: worth_ui_dsl::UiBackdropIdentity,
        portal: UiPortalDeclarationId,
    },
    AmbiguousOrder,
    Cycle,
    ImmediateAdjacencyViolated,
    ReconstructionRequired,
    NoCurrentSnapshot,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiOverlayCommitDenial {
    StalePredecessor,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct UiPreparedOverlayComposition {
    pub(super) predecessor: Option<UiOverlayStackSnapshot>,
    pub(super) snapshot: UiOverlayStackSnapshot,
    pub(super) reservation: UiOverlayReservation,
    pub(super) counters: UiOverlayPlanCounters,
    pub(super) index: UiOverlayDependencyIndex,
    pub(super) relations: UiOverlayRelationCache,
}

impl UiPreparedOverlayComposition {
    pub(crate) fn snapshot(&self) -> &UiOverlayStackSnapshot {
        &self.snapshot
    }
    pub(crate) const fn reservation(&self) -> UiOverlayReservation {
        self.reservation
    }
    pub(crate) const fn counters(&self) -> UiOverlayPlanCounters {
        self.counters
    }
}

pub(crate) struct UiOverlayCompositionInput<'a> {
    pub(super) generation: UiOverlayApplicationGeneration,
    pub(super) presentation: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
    pub(super) extent: &'a UiOverlaySurfaceExtentSnapshot,
    pub(super) portal_snapshot: &'a crate::runtime::portal::UiPortalStackSnapshot,
    pub(super) portal_bindings: &'a [UiOverlayPortalBinding],
    pub(super) motion: Option<&'a UiOverlayMotionSnapshot>,
}

impl<'a> UiOverlayCompositionInput<'a> {
    #[cfg(test)]
    pub(in crate::runtime::overlay_composition) fn new(
        generation: UiOverlayApplicationGeneration,
        presentation: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
        extent: &'a UiOverlaySurfaceExtentSnapshot,
        portal_snapshot: &'a crate::runtime::portal::UiPortalStackSnapshot,
        portal_bindings: &'a [UiOverlayPortalBinding],
        motion: Option<&'a UiOverlayMotionSnapshot>,
    ) -> Self {
        Self {
            generation,
            presentation,
            extent,
            portal_snapshot,
            portal_bindings,
            motion,
        }
    }
}

pub(crate) struct UiOverlayCompositionState {
    pub(super) declarations: Box<[UiBackdropDeclaration]>,
    pub(super) declaration_revision: u64,
    pub(super) capacity: UiOverlayCapacityProfile,
    pub(super) index: Option<UiOverlayDependencyIndex>,
    pub(super) relations: Option<UiOverlayRelationCache>,
    pub(super) current: Option<UiOverlayStackSnapshot>,
}
