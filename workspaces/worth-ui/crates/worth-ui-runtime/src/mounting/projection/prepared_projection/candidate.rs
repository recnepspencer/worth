use super::{
    UiMountedPresentationDeltaSource, UiMountedProjectionDenial, UiMountedProjectionFrame,
    UiMountedProjectionFrameOwner,
};

#[derive(Clone)]
pub struct UiProjectedMountedFrameCandidate {
    pub(in crate::mounting) owner: UiMountedProjectionFrameOwner,
    pub(in crate::mounting) identity_candidate:
        super::super::super::identity_state::UiMountedIdentityFrameCandidate,
    pub(in crate::mounting) projection_changes:
        super::super::super::UiMountedProjectionChangeSnapshot,
    pub(in crate::mounting) presentation_predecessor:
        Option<worth_ui_host_contract::UiMountedFrameIdentity>,
    pub(in crate::mounting) presentation_changed_instances:
        std::rc::Rc<[worth_ui_host_contract::UiMountedInstanceIdentity]>,
    pub(in crate::mounting) presentation_node_changed_instances:
        std::rc::Rc<[worth_ui_host_contract::UiMountedInstanceIdentity]>,
}

impl UiProjectedMountedFrameCandidate {
    pub(in crate::mounting) fn stage_appearance_projection(
        &mut self,
        attempt: crate::runtime::appearance::UiAppearanceProjectionAttempt,
    ) -> Result<(), super::super::frame_storage::UiMountedAppearanceStateMutationDenial> {
        self.owner.stage_appearance_projection(attempt)
    }

    pub(in crate::mounting) fn set_appearance_invalidation_batch(
        &mut self,
        batch: crate::runtime::appearance::UiAppearanceInvalidationBatch,
    ) {
        self.owner.set_appearance_invalidation_batch(batch);
    }

    pub(in crate::mounting) fn clear_appearance_invalidation_batch(&mut self) {
        self.owner.clear_appearance_invalidation_batch();
    }

    pub(in crate::mounting) fn appearance_invalidation_batch(
        &self,
    ) -> Option<crate::runtime::appearance::UiAppearanceInvalidationBatch> {
        self.owner.appearance_invalidation_batch()
    }

    pub(in crate::mounting) fn begin_appearance_lifecycle(
        &mut self,
        session: crate::facade::WorthUiActiveApplicationSessionIdentity,
        generation: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
        graph: crate::graph::UiGraphAuthority<'_>,
    ) {
        self.owner
            .begin_appearance_lifecycle(session, generation, graph);
    }

    pub(in crate::mounting) fn validate_appearance_selection(
        &self,
        batch: &crate::runtime::appearance::UiAppearanceInvalidationBatch,
    ) -> Result<(), UiMountedProjectionDenial> {
        self.owner.validate_appearance_selection(batch)
    }

    pub(in crate::mounting) fn appearance_attempt_inputs(
        &mut self,
        batch: &crate::runtime::appearance::UiAppearanceInvalidationBatch,
    ) -> Result<
        Vec<super::super::frame_storage::UiMountedAppearanceNodeInputContext>,
        UiMountedProjectionDenial,
    > {
        self.owner.appearance_attempt_inputs(batch)
    }

    pub(in crate::mounting) fn reserve_appearance_state(
        &mut self,
        context: &crate::runtime::appearance::UiAppearanceAttemptContext,
    ) -> Result<(), super::super::frame_storage::UiMountedAppearanceStateMutationDenial> {
        self.owner.reserve_appearance_state(context)
    }

    pub(in crate::mounting) fn stage_appearance_input_refresh(
        &mut self,
        node: &super::super::frame_storage::UiMountedAppearanceNodeInputContext,
    ) -> bool {
        self.owner.stage_appearance_input_refresh(node)
    }

    pub(in crate::mounting) fn lower_appearance(
        &mut self,
        presentation: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
        bindings: &[worth_ui_host_contract::UiMountedSurfaceBindingRequirement],
        profile: Option<&worth_ui_host_contract::UiHostAppearanceProfileContract>,
    ) -> Vec<crate::runtime::appearance::UiAppearanceInspectionRecord> {
        self.owner.lower_appearance(presentation, bindings, profile)
    }

    pub(in crate::mounting) fn lower_appearance_with_motion(
        &mut self,
        presentation: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
        bindings: &[worth_ui_host_contract::UiMountedSurfaceBindingRequirement],
        profile: Option<&worth_ui_host_contract::UiHostAppearanceProfileContract>,
        motion: crate::mounting::presentation::UiAcceptedAppearanceMotion,
    ) -> Vec<crate::runtime::appearance::UiAppearanceInspectionRecord> {
        self.owner
            .lower_appearance_with_motion(presentation, bindings, profile, motion)
    }

    pub(in crate::mounting) fn lower_appearance_with_motion_and_overlays(
        &mut self,
        presentation: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
        bindings: &[worth_ui_host_contract::UiMountedSurfaceBindingRequirement],
        profile: Option<&worth_ui_host_contract::UiHostAppearanceProfileContract>,
        motion: crate::mounting::presentation::UiAcceptedAppearanceMotion,
        overlays: &[super::super::UiMountedAppearanceSurfaceOverlayInput],
    ) -> Vec<crate::runtime::appearance::UiAppearanceInspectionRecord> {
        self.owner.lower_appearance_with_motion_and_overlays(
            presentation,
            bindings,
            profile,
            motion,
            overlays,
        )
    }

    pub(in crate::mounting) fn deny_appearance_output(
        &mut self,
    ) -> Vec<crate::runtime::appearance::UiAppearanceInspectionRecord> {
        self.owner.deny_appearance_output()
    }

    pub(in crate::mounting) fn prepare_surface_reconstruction(
        &mut self,
        replacements: &[(
            worth_ui_host_contract::UiSurfaceBindingGeneration,
            crate::mounting::UiSurfaceBindingIdentityView,
        )],
    ) -> Result<(), UiMountedProjectionDenial> {
        {
            let frame = self
                .owner
                .projection_mut_unique()
                .ok_or(UiMountedProjectionDenial::ProjectionOwnerUnavailable)?;
            frame.rebind_retained_mechanics(replacements)?;
            self.presentation_changed_instances =
                frame.mounted_instances().collect::<Vec<_>>().into();
        }
        self.owner.prepare_appearance_reconstruction()?;
        self.presentation_node_changed_instances = self.presentation_changed_instances.clone();
        Ok(())
    }

    pub(in crate::mounting) fn prepare_appearance_reconstruction(
        &mut self,
    ) -> Result<(), UiMountedProjectionDenial> {
        self.owner.prepare_appearance_reconstruction()
    }

    pub(in crate::mounting) fn prepare_appearance_reconstruction_for(
        &mut self,
        bindings: &[worth_ui_host_contract::UiMountedSurfaceBindingRequirement],
    ) -> Result<(), UiMountedProjectionDenial> {
        self.owner.prepare_appearance_reconstruction_for(bindings)
    }

    pub(in crate::mounting) fn appearance_state_capacity_error(
        &self,
    ) -> Option<super::super::frame_storage::UiAppearanceStateCapacityExceeded> {
        self.owner.appearance_state_capacity_error()
    }

    pub(crate) fn appearance_selection_cost_report(
        &self,
    ) -> super::super::UiMountedAppearanceSelectionCostReport {
        self.owner.appearance_selection_cost_report()
    }

    pub(in crate::mounting) fn appearance_motion_targets(
        &self,
        bindings: &[worth_ui_host_contract::UiMountedSurfaceBindingRequirement],
        overlays: &[super::super::UiMountedAppearanceSurfaceOverlayInput],
    ) -> Result<
        Vec<(
            worth_ui_host_contract::UiSemanticSurfaceIdentity,
            worth_ui_host_contract::UiMountedInstanceIdentity,
        )>,
        super::super::frame_storage::UiMountedAppearanceOutputDenial,
    > {
        self.owner.appearance_motion_targets(bindings, overlays)
    }

    pub fn frame(&self) -> &UiMountedProjectionFrame {
        self.owner.projection()
    }

    pub(in crate::mounting) fn projection_rc(&self) -> std::rc::Rc<UiMountedProjectionFrame> {
        self.owner.projection_rc()
    }

    pub(crate) fn presented_receipt_basis(
        &self,
    ) -> &super::super::super::UiMountedNodeReceiptBasis {
        self.identity_candidate.receipt_basis()
    }

    pub(in crate::mounting) fn presentation_delta_source(
        &self,
    ) -> UiMountedPresentationDeltaSource<'_> {
        UiMountedPresentationDeltaSource {
            frame: self.owner.projection(),
            predecessor: self.presentation_predecessor,
            changed_instances: &self.presentation_changed_instances,
            node_changed_instances: &self.presentation_node_changed_instances,
            changes: &self.projection_changes,
        }
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        UiMountedProjectionFrameOwner,
        super::super::super::identity_state::UiMountedIdentityFrameCandidate,
        super::super::super::UiMountedProjectionChangeSnapshot,
    ) {
        (self.owner, self.identity_candidate, self.projection_changes)
    }
}
