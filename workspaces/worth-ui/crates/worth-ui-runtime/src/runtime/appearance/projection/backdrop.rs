use crate::runtime::overlay_composition::UiBackdropInstanceIdentity;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct UiBackdropAppearanceProjection {
    instance: UiBackdropInstanceIdentity,
    declaration: worth_ui_dsl::UiBackdropDeclaration,
    state: super::super::state::UiBackdropAppearanceStateVector,
    role: worth_ui_dsl::UiAppearanceRoleDeclaration,
    theme: super::super::theme::UiThemeResolutionView,
    overlay: super::UiOverlayStackSnapshot,
    aspects: Box<[super::UiResolvedAppearanceAspect]>,
    semantic_digest: u64,
}

impl UiBackdropAppearanceProjection {
    pub(super) fn seal(
        instance: UiBackdropInstanceIdentity,
        declaration: &worth_ui_dsl::UiBackdropDeclaration,
        role: &worth_ui_dsl::UiAppearanceRoleDeclaration,
        state: super::super::state::UiBackdropAppearanceStateVector,
        theme: &super::super::theme::UiThemeResolutionView,
        overlay: super::UiOverlayStackSnapshot,
        aspects: Box<[super::UiResolvedAppearanceAspect]>,
    ) -> Self {
        let mut semantic_digest = 0xcbf2_9ce4_8422_2325_u64;
        semantic_digest = fold(semantic_digest, instance.semantic_digest());
        semantic_digest = fold(semantic_digest, declaration.identity().value());
        semantic_digest = fold_text(semantic_digest, declaration.role().as_str());
        semantic_digest = fold(semantic_digest, declaration.role_revision().value());
        semantic_digest = fold(semantic_digest, declaration.surface().value());
        let semantic_digest = fold_scope(semantic_digest, declaration.scope());
        let semantic_digest = fold_extent(semantic_digest, declaration.extent());
        let semantic_digest = fold_presence(semantic_digest, declaration.presence());
        let semantic_digest = fold_motion(semantic_digest, declaration.motion());
        let semantic_digest = fold_placement(semantic_digest, declaration.placement());
        let semantic_digest = fold(semantic_digest, state.semantic_digest());
        let semantic_digest = fold(semantic_digest, theme.semantic_digest());
        let semantic_digest = fold(semantic_digest, overlay.semantic_digest());
        let semantic_digest = aspects.iter().fold(semantic_digest, |digest, aspect| {
            fold(
                fold(digest, aspect.aspect() as u64 + 1),
                aspect.semantic_digest(),
            )
        });
        Self {
            instance,
            declaration: declaration.clone(),
            state,
            role: role.clone(),
            theme: theme.clone(),
            overlay,
            aspects,
            semantic_digest,
        }
    }

    pub(crate) const fn instance(&self) -> UiBackdropInstanceIdentity {
        self.instance
    }

    pub(crate) const fn declaration(&self) -> &worth_ui_dsl::UiBackdropDeclaration {
        &self.declaration
    }

    pub(crate) const fn state(&self) -> &super::super::state::UiBackdropAppearanceStateVector {
        &self.state
    }

    pub(crate) fn theme(&self) -> &str {
        self.theme.definition_identity()
    }

    pub(crate) const fn theme_revision(&self) -> u64 {
        self.theme.definition_revision()
    }

    pub(crate) const fn catalog_revision(&self) -> u64 {
        self.theme.catalog_revision()
    }

    pub(crate) const fn overlay(&self) -> &super::UiOverlayStackSnapshot {
        &self.overlay
    }

    pub(crate) fn aspects(&self) -> &[super::UiResolvedAppearanceAspect] {
        &self.aspects
    }

    pub(crate) const fn semantic_digest(&self) -> u64 {
        self.semantic_digest
    }

    pub(crate) fn exactly_equivalent(&self, other: &Self) -> bool {
        self == other
    }

    /// Backdrop roles have no owner-state axes. Carry new evidence and overlay
    /// geometry without resolving unchanged, exactly admitted semantic inputs.
    pub(crate) fn reuse_for_overlay(
        &self,
        declaration: &worth_ui_dsl::UiBackdropDeclaration,
        role: &worth_ui_dsl::UiAppearanceRoleDeclaration,
        state: &super::super::state::UiBackdropAppearanceStateVector,
        theme: &super::super::theme::UiThemeResolutionView,
        overlay: &super::UiOverlayStackSnapshot,
    ) -> Option<Self> {
        if self.declaration != *declaration
            || self.role != *role
            || self.theme != *theme
            || self.state.session() != state.session()
            || self.state.generation() != state.generation()
            || self.state.surface() != state.surface()
            || overlay.application() != Some(theme.application().prepared_generation())
            || overlay.surface() != state.surface()
            || overlay.declaration_surface() != declaration.surface()
            || !overlay.contains(self.instance, declaration.identity())
        {
            return None;
        }
        Some(Self::seal(
            self.instance,
            declaration,
            role,
            state.clone(),
            theme,
            overlay.clone(),
            self.aspects.clone(),
        ))
    }
}

fn fold(digest: u64, value: u64) -> u64 {
    digest.wrapping_mul(0x0000_0100_0000_01b3) ^ value
}

fn fold_text(mut digest: u64, value: &str) -> u64 {
    digest = fold(digest, value.len() as u64);
    for byte in value.as_bytes() {
        digest = fold(digest, u64::from(*byte));
    }
    digest
}

fn fold_scope(digest: u64, scope: worth_ui_dsl::UiBackdropScope) -> u64 {
    match scope {
        worth_ui_dsl::UiBackdropScope::SurfaceSingleton => fold(digest, 1),
        worth_ui_dsl::UiBackdropScope::PerPortalInstance(portal) => {
            fold(fold(digest, 2), portal.value())
        }
    }
}

fn fold_extent(digest: u64, extent: worth_ui_dsl::UiBackdropExtentBasis) -> u64 {
    match extent {
        worth_ui_dsl::UiBackdropExtentBasis::SurfaceViewport(surface) => {
            fold(fold(digest, 1), surface.value())
        }
        worth_ui_dsl::UiBackdropExtentBasis::PresentedMosaicRegion { surface, region } => {
            fold(fold(fold(digest, 2), surface.value()), region.value())
        }
    }
}

fn fold_presence(digest: u64, presence: worth_ui_dsl::UiBackdropPresenceBasis) -> u64 {
    match presence {
        worth_ui_dsl::UiBackdropPresenceBasis::Always => fold(digest, 1),
        worth_ui_dsl::UiBackdropPresenceBasis::WhilePortalPresented(portal) => {
            fold(fold(digest, 2), portal.value())
        }
    }
}

fn fold_motion(digest: u64, motion: worth_ui_dsl::UiBackdropMotionBasis) -> u64 {
    match motion {
        worth_ui_dsl::UiBackdropMotionBasis::None => fold(digest, 1),
        worth_ui_dsl::UiBackdropMotionBasis::PortalPresentation(portal) => {
            fold(fold(digest, 2), portal.value())
        }
    }
}

fn fold_placement(digest: u64, placement: worth_ui_dsl::UiBackdropPlacement) -> u64 {
    match placement {
        worth_ui_dsl::UiBackdropPlacement::AboveSurfaceContent => fold(digest, 1),
        worth_ui_dsl::UiBackdropPlacement::ImmediatelyBeforePortal(portal) => {
            fold(fold(digest, 2), portal.value())
        }
        worth_ui_dsl::UiBackdropPlacement::ImmediatelyAfterPortal(portal) => {
            fold(fold(digest, 3), portal.value())
        }
        worth_ui_dsl::UiBackdropPlacement::ImmediatelyBeforeBackdrop(backdrop) => {
            fold(fold(digest, 4), backdrop.value())
        }
        worth_ui_dsl::UiBackdropPlacement::ImmediatelyAfterBackdrop(backdrop) => {
            fold(fold(digest, 5), backdrop.value())
        }
    }
}
