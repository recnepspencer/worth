use worth_ui_dsl::{UiAppearanceRoleIdentity, UiAppearanceRoleRevision, UiAppearanceStateAxis};
use worth_ui_host_contract::{
    UiHostObservationPresentationBasis, UiMountIncarnation, UiMountedInstanceIdentity,
    UiMountedNodeReceiptIdentity, UiSemanticSurfaceIdentity,
};

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
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiAppearanceCoherentBasisInput {
    pub(crate) mounted_identity: crate::mounting::UiMountedIdentityBasis,
    pub(crate) mounted_instance: UiMountedInstanceIdentity,
    pub(crate) node_receipt: UiMountedNodeReceiptIdentity,
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
        let identity = &input.mounted_identity;
        if consumer.graph_node() != identity.graph_node_identity() {
            return Err(UiAppearanceCoherentBasisDenial::ConsumerTargetMismatch);
        }
        validate_mounted_target(mounted, &input)?;
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
        validate_theme(snapshot, themes, surface, &input.theme)?;
        validate_presentation(mounted, input.presentation)?;
        if input.presentation.is_none()
            && (consumer.consumes(UiAppearanceStateAxis::Hover)
                || consumer.consumes(UiAppearanceStateAxis::Pressed))
        {
            let axis = if consumer.consumes(UiAppearanceStateAxis::Hover) {
                UiAppearanceStateAxis::Hover
            } else {
                UiAppearanceStateAxis::Pressed
            };
            return Err(UiAppearanceCoherentBasisDenial::MissingPresentation(axis));
        }
        Ok(Self {
            turn: snapshot.turn(),
            session: snapshot.session(),
            source_basis: snapshot.source_basis(),
            generation: snapshot.generation().clone(),
            consumer: consumer.clone(),
            graph_node: identity.graph_node_identity(),
            mounted_instance: input.mounted_instance,
            incarnation: identity.mount_incarnation(),
            node_receipt: input.node_receipt,
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
                        && posture.node_receipt() == self.node_receipt
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
        digest = fold(digest, self.incarnation.diagnostic_value());
        fold(digest, self.node_receipt.diagnostic_value())
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

    #[cfg(test)]
    #[allow(
        clippy::too_many_arguments,
        reason = "test basis carries the exact identity fields consumed by direct adapters"
    )]
    pub(crate) fn for_test(
        snapshot: &super::UiAppearanceOwnerSnapshot,
        consumer: super::UiAppearanceStateConsumer,
        mounted_instance: UiMountedInstanceIdentity,
        incarnation: UiMountIncarnation,
        node_receipt: UiMountedNodeReceiptIdentity,
        surface: UiSemanticSurfaceIdentity,
        presentation: Option<UiHostObservationPresentationBasis>,
        selection: Option<super::UiAppearanceSelectionSelector>,
        operability_route: Option<Box<str>>,
    ) -> Self {
        Self {
            turn: snapshot.turn(),
            session: snapshot.session(),
            source_basis: snapshot.source_basis(),
            generation: snapshot.generation().clone(),
            graph_node: consumer.graph_node(),
            consumer,
            mounted_instance,
            incarnation,
            node_receipt,
            surface,
            theme: crate::runtime::appearance::UiActiveThemeBinding::for_test(
                surface,
                snapshot.generation().clone(),
            ),
            presentation,
            selection,
            operability_route,
            owner_revisions: owner_revisions(snapshot),
        }
    }
}

fn validate_mounted_target(
    mounted: &crate::mounting::WorthUiMountedSessionState,
    input: &UiAppearanceCoherentBasisInput,
) -> Result<(), UiAppearanceCoherentBasisDenial> {
    if mounted
        .current_mounted_identity_basis(input.mounted_instance)
        .as_ref()
        != Some(&input.mounted_identity)
        || mounted
            .validate_current_receipt(input.mounted_instance, input.node_receipt)
            .is_err()
    {
        return Err(UiAppearanceCoherentBasisDenial::MountedTargetNotCurrent);
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
    presentation: Option<UiHostObservationPresentationBasis>,
) -> Result<(), UiAppearanceCoherentBasisDenial> {
    if presentation.is_some_and(|presentation| {
        mounted
            .validate_current_frame(presentation.frame())
            .is_err()
            || mounted.validate_binding(presentation.binding()).is_err()
            || mounted
                .current_publication()
                .and_then(|publication| publication.semantic_surface_for_presentation(presentation))
                .is_none()
    }) {
        return Err(UiAppearanceCoherentBasisDenial::PresentationNotCurrent);
    }
    Ok(())
}

#[cfg(test)]
pub(crate) fn validate_presentation_for_test(
    mounted: &crate::mounting::WorthUiMountedSessionState,
    presentation: Option<UiHostObservationPresentationBasis>,
) -> Result<(), UiAppearanceCoherentBasisDenial> {
    validate_presentation(mounted, presentation)
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
