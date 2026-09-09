use worth_ui_dsl::{UiAppearanceRoleIdentity, UiAppearanceRoleRevision, UiAppearanceStateAxis};
use worth_ui_host_contract::{
    UiHostObservationPresentationBasis, UiMountIncarnation, UiMountedInstanceIdentity,
    UiMountedNodeReceiptIdentity, UiSemanticSurfaceIdentity,
};

#[path = "coherent_basis_mounted_target.rs"]
mod mounted_target;

#[cfg(test)]
#[path = "coherent_basis_test_support.rs"]
mod test_support;

#[cfg(test)]
pub(crate) use test_support::validate_presentation_for_test;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiAppearanceCoherentBasisDenial {
    ConsumerTargetMismatch,
    AxisNotDemanded(UiAppearanceStateAxis),
    MissingPresentation(UiAppearanceStateAxis),
    ThemeSurfaceMismatch,
    ThemeApplicationMismatch,
    ThemeBindingUnavailable,
    ThemeBindingNotCurrent,
    MountedTargetNotCurrent,
    PresentationNotCurrent,
    OperabilityRouteUnavailable,
    SelectionBindingUnavailable,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiAppearanceCoherentBasisInput {
    pub(crate) mounted_identity: crate::mounting::UiMountedIdentityBasis,
    pub(crate) mounted_instance: UiMountedInstanceIdentity,
    pub(crate) receipt_basis: crate::mounting::UiMountedAppearanceReceiptBasis,
    pub(crate) theme: crate::runtime::appearance::UiActiveThemeBinding,
    pub(crate) presentation: Option<UiHostObservationPresentationBasis>,
    pub(crate) selection: Option<super::UiAppearanceSelectionSelector>,
    pub(crate) operability_route: Option<Box<str>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiAppearanceCoherentBasis {
    turn: crate::runtime::observation::UiObservationTurnIdentity,
    session: crate::facade::WorthUiActiveApplicationSessionIdentity,
    source_basis: u64,
    generation: crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    consumer: super::UiAppearanceStateConsumer,
    graph_node: crate::graph::UiGraphNodeIdentity,
    mounted_instance: UiMountedInstanceIdentity,
    incarnation: UiMountIncarnation,
    owner_node_receipt: Option<UiMountedNodeReceiptIdentity>,
    node_receipt: UiMountedNodeReceiptIdentity,
    surface: UiSemanticSurfaceIdentity,
    theme: crate::runtime::appearance::UiActiveThemeBinding,
    presentation: Option<UiHostObservationPresentationBasis>,
    selection: Option<super::UiAppearanceSelectionSelector>,
    operability_route: Option<Box<str>>,
    owner_revisions: [u64; 6],
}

impl UiAppearanceCoherentBasis {
    pub(crate) fn admit_current(
        snapshot: &super::UiAppearanceOwnerSnapshot,
        consumer: &super::UiAppearanceStateConsumer,
        mounted: &crate::mounting::WorthUiMountedSessionState,
        themes: &crate::runtime::appearance::UiAppearanceThemeState,
        input: UiAppearanceCoherentBasisInput,
    ) -> Result<Self, UiAppearanceCoherentBasisDenial> {
        Self::admit_with_target_validation(snapshot, consumer, mounted, themes, input, None)
    }

    pub(crate) fn admit_prepared(
        frame: &crate::mounting::UiPreparedMountedFrame,
        snapshot: &super::UiAppearanceOwnerSnapshot,
        consumer: &super::UiAppearanceStateConsumer,
        mounted: &crate::mounting::WorthUiMountedSessionState,
        themes: &crate::runtime::appearance::UiAppearanceThemeState,
        input: UiAppearanceCoherentBasisInput,
    ) -> Result<Self, UiAppearanceCoherentBasisDenial> {
        Self::admit_with_target_validation(snapshot, consumer, mounted, themes, input, Some(frame))
    }

    fn admit_with_target_validation(
        snapshot: &super::UiAppearanceOwnerSnapshot,
        consumer: &super::UiAppearanceStateConsumer,
        mounted: &crate::mounting::WorthUiMountedSessionState,
        themes: &crate::runtime::appearance::UiAppearanceThemeState,
        input: UiAppearanceCoherentBasisInput,
        prepared: Option<&crate::mounting::UiPreparedMountedFrame>,
    ) -> Result<Self, UiAppearanceCoherentBasisDenial> {
        let identity = &input.mounted_identity;
        if consumer.graph_node() != identity.graph_node_identity() {
            return Err(UiAppearanceCoherentBasisDenial::ConsumerTargetMismatch);
        }
        if prepared.is_none() {
            mounted_target::validate_current(mounted, &input)?;
        } else {
            mounted_target::validate_prepared(mounted, &input)?;
        }
        for axis in axes() {
            if consumer.consumes(axis) && !snapshot.demand().contains(axis) {
                return Err(UiAppearanceCoherentBasisDenial::AxisNotDemanded(axis));
            }
        }
        if consumer.consumes(UiAppearanceStateAxis::Operability)
            && input.operability_route.is_none()
        {
            return Err(UiAppearanceCoherentBasisDenial::OperabilityRouteUnavailable);
        }
        let surface = identity.semantic_surface_identity();
        mounted_target::validate_selection(mounted, consumer, &input, prepared)?;
        validate_theme(snapshot, themes, surface, &input.theme)?;
        validate_presentation(mounted, surface, input.presentation)?;
        validate_owner_presentation(snapshot, consumer, &input)?;
        Ok(Self {
            turn: snapshot.turn(),
            session: snapshot.session(),
            source_basis: snapshot.source_basis(),
            generation: snapshot.generation().clone(),
            consumer: consumer.clone(),
            graph_node: identity.graph_node_identity(),
            mounted_instance: input.mounted_instance,
            incarnation: identity.mount_incarnation(),
            owner_node_receipt: input.receipt_basis.owner_node_receipt(),
            node_receipt: input.receipt_basis.successor_node_receipt(),
            surface,
            theme: input.theme,
            presentation: input.presentation,
            selection: input.selection,
            operability_route: input.operability_route,
            owner_revisions: owner_revisions(snapshot),
        })
    }

    pub(crate) const fn turn(&self) -> crate::runtime::observation::UiObservationTurnIdentity {
        self.turn
    }

    pub(crate) const fn session(&self) -> crate::facade::WorthUiActiveApplicationSessionIdentity {
        self.session
    }

    pub(crate) const fn source_basis(&self) -> u64 {
        self.source_basis
    }

    pub(crate) fn generation(&self) -> &crate::runtime::WorthUiActiveApplicationGenerationIdentity {
        &self.generation
    }

    pub(crate) const fn graph_node(&self) -> crate::graph::UiGraphNodeIdentity {
        self.graph_node
    }

    pub(crate) const fn mounted_instance(&self) -> UiMountedInstanceIdentity {
        self.mounted_instance
    }

    pub(crate) const fn incarnation(&self) -> UiMountIncarnation {
        self.incarnation
    }

    pub(crate) const fn node_receipt(&self) -> UiMountedNodeReceiptIdentity {
        self.node_receipt
    }

    pub(crate) const fn owner_node_receipt(&self) -> Option<UiMountedNodeReceiptIdentity> {
        self.owner_node_receipt
    }

    pub(crate) const fn surface(&self) -> UiSemanticSurfaceIdentity {
        self.surface
    }

    pub(crate) fn theme(&self) -> &crate::runtime::appearance::UiActiveThemeBinding {
        &self.theme
    }

    pub(crate) const fn presentation(&self) -> Option<UiHostObservationPresentationBasis> {
        self.presentation
    }

    pub(crate) fn consumer(&self) -> &super::UiAppearanceStateConsumer {
        &self.consumer
    }

    pub(crate) const fn selection(&self) -> Option<super::UiAppearanceSelectionSelector> {
        self.selection
    }

    pub(crate) fn operability_route(&self) -> Option<&str> {
        self.operability_route.as_deref()
    }

    pub(crate) const fn role(&self) -> &UiAppearanceRoleIdentity {
        self.consumer.role()
    }

    pub(crate) const fn role_revision(&self) -> UiAppearanceRoleRevision {
        self.consumer.role_revision()
    }

    pub(crate) fn matches_snapshot(&self, snapshot: &super::UiAppearanceOwnerSnapshot) -> bool {
        self.turn == snapshot.turn()
            && self.session == snapshot.session()
            && self.source_basis == snapshot.source_basis()
            && self.generation == *snapshot.generation()
    }

    pub(crate) fn presentation_matches_owner_snapshot(
        &self,
        snapshot: &super::UiAppearanceOwnerSnapshot,
    ) -> bool {
        if self.consumer.consumes(UiAppearanceStateAxis::Hover) {
            let Some(pointer_owner) = snapshot.pointer_presence() else {
                return true;
            };
            let Some(pointer) = pointer_owner.primary_pointer(self.surface) else {
                return true;
            };
            let Some(posture) = pointer_owner
                .postures()
                .iter()
                .find(|posture| posture.pointer() == pointer)
            else {
                return false;
            };
            if self.presentation != Some(posture.presentation()) {
                return false;
            }
        }
        if self.consumer.consumes(UiAppearanceStateAxis::Pressed) {
            if let Some(pressed_owner) = snapshot.pressed() {
                let mismatch = pressed_owner.postures().iter().any(|posture| {
                    posture.target() == self.mounted_instance
                        && Some(posture.node_receipt()) == self.owner_node_receipt
                        && self.presentation != Some(posture.presentation())
                });
                if mismatch {
                    return false;
                }
            }
        }
        true
    }

    pub(crate) const fn owner_revisions(&self) -> &[u64; 6] {
        &self.owner_revisions
    }

    pub(crate) fn semantic_digest(&self) -> u64 {
        let mut digest = 0xcbf2_9ce4_8422_2325_u64;
        digest = fold(digest, self.session.as_u64());
        digest = fold(
            digest,
            self.generation
                .prepared_generation()
                .semantic_package_identity()
                .narrowing_fingerprint(),
        );
        digest = fold(digest, self.surface.diagnostic_value());
        digest = fold(digest, self.graph_node.digest());
        digest = fold(digest, self.mounted_instance.diagnostic_value());
        fold(digest, self.incarnation.diagnostic_value())
    }

    pub(crate) fn evidence_digest(&self) -> u64 {
        let mut digest = self.semantic_digest();
        digest = fold(digest, self.turn.as_u64());
        digest = fold(digest, self.source_basis);
        for revision in self.owner_revisions {
            digest = fold(digest, revision);
        }
        digest
    }
}

fn validate_owner_presentation(
    snapshot: &super::UiAppearanceOwnerSnapshot,
    consumer: &super::UiAppearanceStateConsumer,
    input: &UiAppearanceCoherentBasisInput,
) -> Result<(), UiAppearanceCoherentBasisDenial> {
    if input.presentation.is_some() {
        return Ok(());
    }
    if consumer.consumes(UiAppearanceStateAxis::Hover)
        && snapshot.pointer_presence().is_some_and(|owner| {
            owner
                .primary_pointer(input.mounted_identity.semantic_surface_identity())
                .is_some()
        })
    {
        return Err(UiAppearanceCoherentBasisDenial::MissingPresentation(
            UiAppearanceStateAxis::Hover,
        ));
    }
    if consumer.consumes(UiAppearanceStateAxis::Pressed)
        && snapshot.pressed().is_some_and(|owner| {
            owner.postures().iter().any(|posture| {
                posture.target() == input.mounted_instance
                    && Some(posture.node_receipt()) == input.receipt_basis.owner_node_receipt()
            })
        })
    {
        return Err(UiAppearanceCoherentBasisDenial::MissingPresentation(
            UiAppearanceStateAxis::Pressed,
        ));
    }
    Ok(())
}

fn validate_theme(
    snapshot: &super::UiAppearanceOwnerSnapshot,
    themes: &crate::runtime::appearance::UiAppearanceThemeState,
    surface: UiSemanticSurfaceIdentity,
    theme: &crate::runtime::appearance::UiActiveThemeBinding,
) -> Result<(), UiAppearanceCoherentBasisDenial> {
    if theme.surface() != surface || theme.capability().surface() != surface {
        return Err(UiAppearanceCoherentBasisDenial::ThemeSurfaceMismatch);
    }
    if theme.capability().application() != snapshot.generation() {
        return Err(UiAppearanceCoherentBasisDenial::ThemeApplicationMismatch);
    }
    if theme.binding_generation() == 0 {
        return Err(UiAppearanceCoherentBasisDenial::ThemeBindingUnavailable);
    }
    if themes.active_binding(surface) != Some(theme) {
        return Err(UiAppearanceCoherentBasisDenial::ThemeBindingNotCurrent);
    }
    Ok(())
}

fn validate_presentation(
    mounted: &crate::mounting::WorthUiMountedSessionState,
    surface: UiSemanticSurfaceIdentity,
    presentation: Option<UiHostObservationPresentationBasis>,
) -> Result<(), UiAppearanceCoherentBasisDenial> {
    if presentation.is_some_and(|presentation| {
        mounted
            .current_semantic_surface_for_presentation(presentation)
            .ok()
            != Some(surface)
    }) {
        return Err(UiAppearanceCoherentBasisDenial::PresentationNotCurrent);
    }
    Ok(())
}

fn owner_revisions(snapshot: &super::UiAppearanceOwnerSnapshot) -> [u64; 6] {
    [
        snapshot
            .operability()
            .map_or(0, |value| value.owner_revision()),
        snapshot.focus().map_or(0, |value| value.owner_revision()),
        snapshot
            .validation()
            .map_or(0, |value| value.owner_revision()),
        snapshot
            .selection()
            .map_or(0, |value| value.owner_revision()),
        snapshot
            .pointer_presence()
            .map_or(0, |value| value.owner_revision()),
        snapshot.pressed().map_or(0, |value| value.owner_revision()),
    ]
}

fn fold(digest: u64, value: u64) -> u64 {
    digest.wrapping_mul(0x0000_0100_0000_01b3) ^ value
}

const fn axes() -> [UiAppearanceStateAxis; 6] {
    [
        UiAppearanceStateAxis::Operability,
        UiAppearanceStateAxis::Focus,
        UiAppearanceStateAxis::Validation,
        UiAppearanceStateAxis::Selection,
        UiAppearanceStateAxis::Hover,
        UiAppearanceStateAxis::Pressed,
    ]
}
