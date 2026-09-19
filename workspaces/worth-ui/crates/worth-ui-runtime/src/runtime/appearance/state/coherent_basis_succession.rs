use super::{
    UiAppearanceCoherentBasis, UiAppearanceCoherentBasisDenial, UiAppearanceCoherentBasisInput,
};

impl UiAppearanceCoherentBasis {
    pub(crate) fn admit_generation_succession(
        frame: &crate::mounting::UiAssembledMountedFrame,
        snapshot: &super::super::UiAppearanceOwnerSnapshot,
        consumer: &super::super::UiAppearanceStateConsumer,
        mounted: &crate::mounting::UiMountedGraphReplacementSuccessor,
        themes: &crate::runtime::presentation_state::UiPreparedAppearanceGenerationSuccession,
        input: UiAppearanceCoherentBasisInput,
    ) -> Result<Self, UiAppearanceCoherentBasisDenial> {
        for axis in super::axes() {
            if consumer.consumes(axis) && !snapshot.demand().contains(axis) {
                return Err(UiAppearanceCoherentBasisDenial::AxisNotDemanded(axis));
            }
        }
        if consumer.consumes(worth_ui_dsl::UiAppearanceStateAxis::Operability)
            && input.operability_route.is_none()
        {
            return Err(UiAppearanceCoherentBasisDenial::OperabilityRouteUnavailable);
        }
        if consumer.consumes(worth_ui_dsl::UiAppearanceStateAxis::Selection)
            && input.selection.is_none()
        {
            return Err(UiAppearanceCoherentBasisDenial::SelectionBindingUnavailable);
        }
        if frame.generation() != themes.successor().prepared_generation()
            || snapshot.generation() != themes.successor()
            || snapshot.succession_predecessor() != Some(themes.predecessor())
            || snapshot.session() != themes.successor().session_identity()
            || input.theme.capability().application() != themes.successor()
        {
            return Err(UiAppearanceCoherentBasisDenial::ThemeApplicationMismatch);
        }
        if frame
            .presented_receipt_basis()
            .receipt_for(input.mounted_instance)
            != Some(input.receipt_basis.successor_node_receipt())
        {
            return Err(UiAppearanceCoherentBasisDenial::MountedTargetNotCurrent);
        }
        let identity = &input.mounted_identity;
        let surface = identity.semantic_surface_identity();
        if consumer.graph_node() != identity.graph_node_identity() {
            return Err(UiAppearanceCoherentBasisDenial::ConsumerTargetMismatch);
        }
        if mounted
            .appearance_identity_basis(input.mounted_instance)
            .as_ref()
            != Some(identity)
            || input.receipt_basis.mounted_instance() != input.mounted_instance
            || input.receipt_basis.incarnation() != identity.mount_incarnation()
            || mounted
                .validate_appearance_receipt_basis(input.receipt_basis)
                .is_err()
        {
            return Err(UiAppearanceCoherentBasisDenial::MountedTargetNotCurrent);
        }
        if input.theme.surface() != surface || input.theme.capability().surface() != surface {
            return Err(UiAppearanceCoherentBasisDenial::ThemeSurfaceMismatch);
        }
        if themes.binding(surface) != Some(&input.theme) || input.theme.binding_generation() == 0 {
            return Err(UiAppearanceCoherentBasisDenial::ThemeBindingNotCurrent);
        }
        Ok(Self {
            turn: snapshot.turn(),
            session: snapshot.session(),
            source_basis: snapshot.source_basis(),
            source_generation: snapshot.generation().clone(),
            generation: themes.successor().clone(),
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
            owner_revisions: super::owner_revisions(snapshot),
        })
    }
}
