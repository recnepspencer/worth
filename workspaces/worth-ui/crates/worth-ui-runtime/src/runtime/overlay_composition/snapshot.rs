use worth_ui_dsl::{
    UiBackdropDeclaration, UiBackdropIdentity, UiBackdropPlacement, UiBackdropPresenceBasis,
    UiBackdropScope, UiPortalDeclarationId, UiSemanticSurfaceDeclarationIdentity,
};

#[path = "snapshot_digest.rs"]
mod snapshot_digest;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum UiOverlayApplicationGeneration {
    Prepared(
        crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationIdentity,
    ),
    #[cfg(test)]
    Test(u64),
}

impl UiOverlayApplicationGeneration {
    pub(crate) fn from_prepared(
        identity: crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationIdentity,
    ) -> Self {
        Self::Prepared(identity)
    }

    #[cfg(test)]
    pub(crate) const fn for_test(value: u64) -> Self {
        Self::Test(value)
    }

}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum UiOverlayBackdropInstanceScope {
    SurfaceSingleton,
    Portal(crate::runtime::portal::UiPortalIdentity),
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct UiBackdropInstanceIdentity {
    declaration: UiBackdropIdentity,
    scope: UiOverlayBackdropInstanceScope,
}

impl UiBackdropInstanceIdentity {
    pub(in crate::runtime::overlay_composition) const fn new(
        declaration: UiBackdropIdentity,
        scope: UiOverlayBackdropInstanceScope,
    ) -> Self {
        Self { declaration, scope }
    }

    #[cfg(test)]
    pub(crate) const fn surface_singleton(declaration: UiBackdropIdentity) -> Self {
        Self::new(
            declaration,
            UiOverlayBackdropInstanceScope::SurfaceSingleton,
        )
    }

    pub(crate) const fn declaration(self) -> UiBackdropIdentity {
        self.declaration
    }

    pub(crate) const fn scope(self) -> UiOverlayBackdropInstanceScope {
        self.scope
    }

}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum UiOverlayParticipantIdentity {
    Portal(crate::runtime::portal::UiPortalIdentity),
    Backdrop(UiBackdropInstanceIdentity),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum UiOverlayExtent {
    SurfaceViewport {
        basis: worth_ui_dsl::UiBackdropExtentBasis,
        bounds: worth_ui_host_contract::UiMountedCanonicalBox,
    },
    PresentedMosaicRegion {
        basis: worth_ui_dsl::UiBackdropExtentBasis,
        bounds: worth_ui_host_contract::UiMountedCanonicalBox,
    },
}

impl UiOverlayExtent {
    pub(crate) const fn basis(self) -> worth_ui_dsl::UiBackdropExtentBasis {
        match self {
            Self::SurfaceViewport { basis, .. } | Self::PresentedMosaicRegion { basis, .. } => {
                basis
            }
        }
    }

    pub(crate) const fn bounds(self) -> worth_ui_host_contract::UiMountedCanonicalBox {
        match self {
            Self::SurfaceViewport { bounds, .. } | Self::PresentedMosaicRegion { bounds, .. } => {
                bounds
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct UiOverlayPortalRow {
    declaration: UiPortalDeclarationId,
    portal: crate::runtime::portal::UiPortalIdentity,
    parent: Option<crate::runtime::portal::UiPortalIdentity>,
    ordinal: crate::runtime::portal::UiPortalStackOrdinal,
    lifecycle: crate::runtime::portal::UiPortalLifecyclePosture,
}

impl UiOverlayPortalRow {
    pub(in crate::runtime::overlay_composition) const fn new(
        declaration: UiPortalDeclarationId,
        portal: crate::runtime::portal::UiPortalIdentity,
        parent: Option<crate::runtime::portal::UiPortalIdentity>,
        ordinal: crate::runtime::portal::UiPortalStackOrdinal,
        lifecycle: crate::runtime::portal::UiPortalLifecyclePosture,
    ) -> Self {
        Self {
            declaration,
            portal,
            parent,
            ordinal,
            lifecycle,
        }
    }

    pub(crate) const fn declaration(&self) -> UiPortalDeclarationId {
        self.declaration
    }

    pub(crate) const fn portal(&self) -> crate::runtime::portal::UiPortalIdentity {
        self.portal
    }

    pub(crate) const fn parent(&self) -> Option<crate::runtime::portal::UiPortalIdentity> {
        self.parent
    }

    pub(crate) const fn ordinal(&self) -> crate::runtime::portal::UiPortalStackOrdinal {
        self.ordinal
    }

    pub(crate) const fn lifecycle(&self) -> crate::runtime::portal::UiPortalLifecyclePosture {
        self.lifecycle
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct UiOverlayBackdropRow {
    identity: UiBackdropInstanceIdentity,
    declaration: UiBackdropIdentity,
    scope: UiBackdropScope,
    extent: UiOverlayExtent,
    presence: UiBackdropPresenceBasis,
    placement: UiBackdropPlacement,
    motion: Option<super::extent::UiOverlayMotionBinding>,
    role: worth_ui_dsl::UiAppearanceRoleIdentity,
    role_revision: worth_ui_dsl::UiAppearanceRoleRevision,
}

impl UiOverlayBackdropRow {
    pub(in crate::runtime::overlay_composition) fn new(
        identity: UiBackdropInstanceIdentity,
        declaration: &UiBackdropDeclaration,
        extent: UiOverlayExtent,
        motion: Option<super::extent::UiOverlayMotionBinding>,
    ) -> Self {
        Self {
            identity,
            declaration: declaration.identity(),
            scope: declaration.scope(),
            extent,
            presence: declaration.presence(),
            placement: declaration.placement(),
            motion,
            role: declaration.role().clone(),
            role_revision: declaration.role_revision(),
        }
    }

    #[cfg(test)]
    pub(crate) fn for_test(
        identity: UiBackdropInstanceIdentity,
        declaration: &UiBackdropDeclaration,
        extent: UiOverlayExtent,
    ) -> Self {
        Self::new(identity, declaration, extent, None)
    }

    pub(crate) const fn identity(&self) -> UiBackdropInstanceIdentity {
        self.identity
    }

    pub(crate) const fn declaration(&self) -> UiBackdropIdentity {
        self.declaration
    }

    pub(crate) const fn scope(&self) -> UiBackdropScope {
        self.scope
    }

    pub(crate) const fn extent(&self) -> UiOverlayExtent {
        self.extent
    }

    pub(crate) const fn presence(&self) -> UiBackdropPresenceBasis {
        self.presence
    }

    pub(crate) const fn placement(&self) -> UiBackdropPlacement {
        self.placement
    }

    pub(crate) fn motion(&self) -> Option<super::extent::UiOverlayMotionBinding> {
        self.motion
    }

    pub(crate) fn role(&self) -> &worth_ui_dsl::UiAppearanceRoleIdentity {
        &self.role
    }

    pub(crate) const fn role_revision(&self) -> worth_ui_dsl::UiAppearanceRoleRevision {
        self.role_revision
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum UiOverlayStackParticipant {
    Portal(UiOverlayPortalRow),
    Backdrop(UiOverlayBackdropRow),
}

impl UiOverlayStackParticipant {
    pub(crate) const fn identity(&self) -> UiOverlayParticipantIdentity {
        match self {
            Self::Portal(row) => UiOverlayParticipantIdentity::Portal(row.portal()),
            Self::Backdrop(row) => UiOverlayParticipantIdentity::Backdrop(row.identity()),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct UiOverlayStackSnapshot {
    generation: UiOverlayApplicationGeneration,
    declaration_surface: UiSemanticSurfaceDeclarationIdentity,
    runtime_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    presentation: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
    portal_revision: u64,
    backdrop_declaration_revision: u64,
    extent_revision: u64,
    motion_revision: Option<u64>,
    participants: Box<[UiOverlayStackParticipant]>,
}

impl UiOverlayStackSnapshot {
    pub(in crate::runtime::overlay_composition) fn seal(
        generation: UiOverlayApplicationGeneration,
        declaration_surface: UiSemanticSurfaceDeclarationIdentity,
        runtime_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        presentation: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
        portal_revision: u64,
        backdrop_declaration_revision: u64,
        extent_revision: u64,
        motion_revision: Option<u64>,
        participants: impl IntoIterator<Item = UiOverlayStackParticipant>,
    ) -> Self {
        Self {
            generation,
            declaration_surface,
            runtime_surface,
            presentation,
            portal_revision,
            backdrop_declaration_revision,
            extent_revision,
            motion_revision,
            participants: participants.into_iter().collect(),
        }
    }

    #[cfg(test)]
    pub(crate) fn seal_for_test(
        generation: UiOverlayApplicationGeneration,
        declaration_surface: UiSemanticSurfaceDeclarationIdentity,
        runtime_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        presentation: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
        portal_revision: u64,
        backdrop_declaration_revision: u64,
        extent_revision: u64,
        motion_revision: Option<u64>,
        participants: impl IntoIterator<Item = UiOverlayStackParticipant>,
    ) -> Self {
        Self::seal(
            generation,
            declaration_surface,
            runtime_surface,
            presentation,
            portal_revision,
            backdrop_declaration_revision,
            extent_revision,
            motion_revision,
            participants,
        )
    }

    pub(crate) fn generation(&self) -> &UiOverlayApplicationGeneration {
        &self.generation
    }

    pub(crate) const fn declaration_surface(&self) -> UiSemanticSurfaceDeclarationIdentity {
        self.declaration_surface
    }

    pub(crate) const fn runtime_surface(
        &self,
    ) -> worth_ui_host_contract::UiSemanticSurfaceIdentity {
        self.runtime_surface
    }

    pub(crate) const fn presentation(
        &self,
    ) -> worth_ui_host_contract::UiMountedPresentationAttemptIdentity {
        self.presentation
    }

    pub(crate) const fn portal_revision(&self) -> u64 {
        self.portal_revision
    }

    pub(crate) const fn backdrop_declaration_revision(&self) -> u64 {
        self.backdrop_declaration_revision
    }

    pub(crate) const fn extent_revision(&self) -> u64 {
        self.extent_revision
    }

    pub(crate) const fn motion_revision(&self) -> Option<u64> {
        self.motion_revision
    }

    pub(crate) fn participants(&self) -> &[UiOverlayStackParticipant] {
        &self.participants
    }

}
