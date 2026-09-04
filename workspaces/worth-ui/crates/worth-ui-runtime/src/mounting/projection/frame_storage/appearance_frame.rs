use super::{appearance_state, UiMountedProjectionFrame, UiMountedProjectionFrameOwner};

impl UiMountedProjectionFrame {
    pub(crate) fn appearance_node_inputs_for_reconstruction(
        &self,
    ) -> Vec<super::UiMountedAppearanceNodeInputContext> {
        self.semantic
            .nodes_in_mounted_order()
            .map(|node| {
                let receipt = node.receipt();
                let node_receipt = self
                    .receipt_basis
                    .receipt_for(receipt.mounted_instance())
                    .expect("mounted projection node belongs to its receipt basis");
                super::UiMountedAppearanceNodeInputContext {
                    frame: self.frame,
                    semantic_surface: receipt.semantic_surface(),
                    mounted_instance: receipt.mounted_instance(),
                    graph_node: receipt.graph_node(),
                    incarnation: receipt.incarnation(),
                    node_receipt,
                    issuer: self.receipt_basis.issuer(),
                    plan_digest: receipt.plan_digest(),
                    allocation: receipt.allocation(),
                }
            })
            .collect()
    }
}

impl UiMountedProjectionFrameOwner {
    pub(crate) fn set_appearance_invalidation_batch(
        &mut self,
        batch: crate::runtime::appearance::UiAppearanceInvalidationBatch,
    ) {
        self.appearance.set_batch(batch);
    }

    pub(crate) fn appearance_invalidation_batch(
        &self,
    ) -> Option<crate::runtime::appearance::UiAppearanceInvalidationBatch> {
        self.appearance.batch().cloned()
    }

    pub(crate) fn begin_appearance_lifecycle(
        &mut self,
        session: crate::facade::WorthUiActiveApplicationSessionIdentity,
        generation: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    ) {
        let retired_instances = self.appearance.retired_instances().to_vec();
        let retired_memberships =
            self.appearance
                .begin_epoch(session, generation, &retired_instances);
        self.appearance
            .record_lifecycle_memberships_retired(retired_memberships);
    }

    pub(crate) fn validate_appearance_selection(
        &self,
        batch: &crate::runtime::appearance::UiAppearanceInvalidationBatch,
    ) -> Result<(), super::super::UiMountedProjectionDenial> {
        self.appearance.validate_selection(batch)
    }

    pub(crate) fn appearance_attempt_inputs(
        &mut self,
        batch: &crate::runtime::appearance::UiAppearanceInvalidationBatch,
    ) -> Result<
        Vec<super::UiMountedAppearanceNodeInputContext>,
        super::super::UiMountedProjectionDenial,
    > {
        self.appearance.validate_selection(batch)?;
        let contexts = self
            .appearance
            .selected_instances()
            .iter()
            .copied()
            .map(|instance| {
                let node = self.projection.semantic_projection().node(instance).ok_or(
                    super::super::UiMountedProjectionDenial::AppearanceSelectionFrameMismatch,
                )?;
                let receipt = node.receipt();
                let node_receipt = self.projection.receipt_basis.receipt_for(instance).ok_or(
                    super::super::UiMountedProjectionDenial::AppearanceSelectionFrameMismatch,
                )?;
                Ok(super::UiMountedAppearanceNodeInputContext {
                    frame: self.projection.frame_identity(),
                    semantic_surface: receipt.semantic_surface(),
                    mounted_instance: instance,
                    graph_node: receipt.graph_node(),
                    incarnation: receipt.incarnation(),
                    node_receipt,
                    issuer: self.projection.receipt_basis.issuer(),
                    plan_digest: receipt.plan_digest(),
                    allocation: receipt.allocation(),
                })
            })
            .collect::<Result<Vec<_>, super::super::UiMountedProjectionDenial>>()?;
        self.appearance.record_materialized_contexts(contexts.len());
        Ok(contexts)
    }

    pub(crate) fn appearance_selection_cost_report(
        &self,
    ) -> super::super::UiMountedAppearanceSelectionCostReport {
        self.appearance.selection_cost_report()
    }

    pub(crate) fn prepare_appearance_reconstruction(&mut self) {
        let nodes = self.projection.appearance_node_inputs_for_reconstruction();
        self.appearance.prepare_reconstruction(nodes);
    }

    pub(crate) fn reserve_appearance_state(
        &mut self,
        context: &crate::runtime::appearance::UiAppearanceAttemptContext,
    ) -> Result<(), appearance_state::UiMountedAppearanceStateMutationDenial> {
        self.appearance.reserve(context)
    }

    pub(crate) fn stage_appearance_projection(
        &mut self,
        attempt: crate::runtime::appearance::UiAppearanceProjectionAttempt,
    ) -> Result<(), appearance_state::UiMountedAppearanceStateMutationDenial> {
        self.appearance.stage(attempt)
    }

    pub(crate) const fn appearance_state_capacity_error(
        &self,
    ) -> Option<appearance_state::UiAppearanceStateCapacityExceeded> {
        self.appearance.capacity_error()
    }

    pub(crate) fn lower_appearance(
        &mut self,
        presentation: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
    ) -> Vec<crate::runtime::appearance::UiAppearanceInspectionRecord> {
        self.appearance.lower(presentation)
    }
}
