use super::{appearance_state, UiMountedProjectionFrame, UiMountedProjectionFrameOwner};

impl UiMountedProjectionFrame {
    fn appearance_attached_instances(
        &self,
    ) -> std::collections::BTreeSet<worth_ui_host_contract::UiMountedInstanceIdentity> {
        self.semantic
            .nodes_in_mounted_order()
            .filter(|node| node.has_appearance_attachment)
            .map(|node| node.receipt().mounted_instance())
            .collect()
    }

    pub(crate) fn portal_has_appearance_attachment(
        &self,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) -> Result<bool, super::super::UiMountedProjectionDenial> {
        let node = self
            .semantic
            .node(instance)
            .ok_or(super::super::UiMountedProjectionDenial::AppearanceSelectionFrameMismatch)?;
        if node.receipt().semantic_surface() != surface {
            return Err(super::super::UiMountedProjectionDenial::AppearanceSelectionFrameMismatch);
        }
        Ok(node.has_appearance_attachment)
    }

    pub(super) fn appearance_node_input(
        &self,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) -> Result<super::UiMountedAppearanceNodeInputContext, super::super::UiMountedProjectionDenial>
    {
        let node = self
            .semantic
            .node(instance)
            .ok_or(super::super::UiMountedProjectionDenial::AppearanceSelectionFrameMismatch)?;
        let receipt = node.receipt();
        let node_receipt = self
            .receipt_basis
            .receipt_for(instance)
            .ok_or(super::super::UiMountedProjectionDenial::AppearanceSelectionFrameMismatch)?;
        Ok(super::UiMountedAppearanceNodeInputContext {
            frame: self.frame,
            semantic_surface: receipt.semantic_surface(),
            mounted_instance: instance,
            graph_node: receipt.graph_node(),
            incarnation: receipt.incarnation(),
            node_receipt,
            issuer: self.receipt_basis.issuer(),
            plan_digest: receipt.plan_digest(),
            allocation: node.completed_appearance_geometry().allocation,
            appearance_clip: node.completed_appearance_geometry().clip,
            surface_paint_order: node.surface_paint_order,
            geometry_input: self
                .semantic
                .surface_for(receipt.semantic_surface())
                .map(|surface| {
                    super::super::appearance::UiMountedAppearanceGeometryInput::from_node(
                        node,
                        surface.binding,
                    )
                }),
            text_foreground_spans: self.appearance_visible_foreground_spans(instance).map_err(
                |_| super::super::UiMountedProjectionDenial::AppearanceTextCandidatesUnavailable,
            )?,
        })
    }
}

impl UiMountedProjectionFrameOwner {
    pub(crate) fn appearance_raw_opacity_for_instance(
        &self,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) -> Option<worth_ui_host_contract::UiMountedAppearanceOpacity> {
        self.appearance.raw_opacity_for_instance(instance)
    }

    pub(crate) fn appearance_changed_instances(
        &self,
    ) -> &[worth_ui_host_contract::UiMountedInstanceIdentity] {
        self.appearance.selected_instances()
    }

    pub(crate) fn retired_appearance_instances(
        &self,
    ) -> &[worth_ui_host_contract::UiMountedInstanceIdentity] {
        self.appearance.retired_instances()
    }

    pub(crate) fn stage_appearance_input_refresh(
        &mut self,
        node: &super::UiMountedAppearanceNodeInputContext,
    ) -> bool {
        self.appearance.stage_input_refresh(node)
    }

    pub(crate) fn appearance_order_retained_bytes(&self) -> usize {
        self.appearance.order_retained_bytes()
    }
    #[cfg(test)]
    pub(crate) fn qualified_outline_fringe_for_test(
        &self,
        bindings: &[worth_ui_host_contract::UiMountedSurfaceBindingRequirement],
        profile: Option<&worth_ui_host_contract::UiHostAppearanceProfileContract>,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) -> Result<
        worth_ui_host_contract::UiAppearanceLogicalLength,
        crate::mounting::UiMountedAppearanceLoweringDenial,
    > {
        super::super::appearance::UiMountedAppearanceGeometryScope::new(bindings, profile)
            .outline_fringe(surface)
    }

    #[cfg(test)]
    pub(crate) fn clone_for_appearance_output_test(&self) -> Self {
        let mut owner = self.clone();
        owner.appearance.isolate_measurements();
        owner
    }

    pub(crate) fn set_appearance_invalidation_batch(
        &mut self,
        batch: crate::runtime::appearance::UiAppearanceInvalidationBatch,
    ) {
        self.appearance.set_batch(batch);
    }

    pub(crate) fn clear_appearance_invalidation_batch(&mut self) {
        self.appearance.clear_batch();
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
        _graph: crate::graph::UiGraphAuthority<'_>,
    ) {
        // The prepared-frame boundary has already matched `graph` to this
        // projection. Detachment is occurrence-owned, so use the current
        // mounted rows rather than predecessor graph-node identities.
        let detached = if self
            .appearance
            .requires_epoch_transition(session, generation)
        {
            let attached_instances = self.projection.appearance_attached_instances();
            self.appearance.retire_detached_on_epoch_change(
                session,
                generation,
                &attached_instances,
            )
        } else {
            0
        };
        let retired_instances = self.appearance.retired_instances().to_vec();
        let retired_memberships =
            self.appearance
                .begin_epoch(session, generation, &retired_instances);
        self.appearance
            .record_lifecycle_memberships_retired(retired_memberships.saturating_add(detached));
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
            .map(|instance| self.projection.appearance_node_input(instance))
            .collect::<Result<Vec<_>, super::super::UiMountedProjectionDenial>>()?;
        self.appearance.record_materialized_contexts(contexts.len());
        Ok(contexts)
    }

    pub(crate) fn appearance_selection_cost_report(
        &self,
    ) -> super::super::UiMountedAppearanceSelectionCostReport {
        self.appearance.selection_cost_report()
    }

    pub(crate) fn appearance_motion_targets(
        &self,
        bindings: &[worth_ui_host_contract::UiMountedSurfaceBindingRequirement],
        overlays: &[crate::mounting::UiMountedAppearanceSurfaceOverlayInput],
    ) -> Result<
        Vec<(
            worth_ui_host_contract::UiSemanticSurfaceIdentity,
            worth_ui_host_contract::UiMountedInstanceIdentity,
        )>,
        super::UiMountedAppearanceOutputDenial,
    > {
        let mut appearance = self.appearance.clone();
        appearance.stage_portal_ownership_changes(&self.projection, bindings, overlays)?;
        let mut targets = appearance
            .selected_instances()
            .iter()
            .copied()
            .map(|instance| {
                self.projection
                    .appearance_node_input(instance)
                    .map(|context| (context.semantic_surface, instance))
                    .map_err(|_| {
                        super::UiMountedAppearanceOutputDenial::CurrentProjectionUnavailable
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        targets.extend(overlays.iter().flat_map(|overlay| {
            overlay
                .backdrops
                .iter()
                .filter_map(|backdrop| backdrop.motion_target)
                .map(|instance| (overlay.semantic_surface, instance))
        }));
        targets.sort_unstable();
        targets.dedup();
        Ok(targets)
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
        bindings: &[worth_ui_host_contract::UiMountedSurfaceBindingRequirement],
        profile: Option<&worth_ui_host_contract::UiHostAppearanceProfileContract>,
    ) -> Vec<crate::runtime::appearance::UiAppearanceInspectionRecord> {
        self.lower_appearance_with_motion(
            presentation,
            bindings,
            profile,
            crate::mounting::presentation::UiAcceptedAppearanceMotion::default(),
        )
    }

    pub(crate) fn lower_appearance_with_motion(
        &mut self,
        presentation: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
        bindings: &[worth_ui_host_contract::UiMountedSurfaceBindingRequirement],
        profile: Option<&worth_ui_host_contract::UiHostAppearanceProfileContract>,
        motion: crate::mounting::presentation::UiAcceptedAppearanceMotion,
    ) -> Vec<crate::runtime::appearance::UiAppearanceInspectionRecord> {
        self.lower_appearance_with_motion_and_overlays(presentation, bindings, profile, motion, &[])
    }

    pub(crate) fn lower_appearance_with_motion_and_overlays(
        &mut self,
        presentation: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
        bindings: &[worth_ui_host_contract::UiMountedSurfaceBindingRequirement],
        profile: Option<&worth_ui_host_contract::UiHostAppearanceProfileContract>,
        motion: crate::mounting::presentation::UiAcceptedAppearanceMotion,
        overlays: &[crate::mounting::UiMountedAppearanceSurfaceOverlayInput],
    ) -> Vec<crate::runtime::appearance::UiAppearanceInspectionRecord> {
        let mut candidate = self.appearance.clone();
        let portal_instances = super::appearance_state::portal_instances(overlays);
        if let Err(_denial) =
            candidate.stage_portal_ownership_changes(&self.projection, bindings, overlays)
        {
            self.unpublished_appearance =
                Err(super::UiMountedAppearanceOutputDenial::CurrentProjectionUnavailable);
            return self.appearance.reject_unpublished_output(Vec::new());
        }
        let geometry =
            crate::mounting::projection::appearance::UiMountedAppearanceGeometryScope::with_motion(
                bindings, profile, motion,
            );
        let records = match candidate.lower_with_portals(presentation, &geometry, &portal_instances)
        {
            Ok(records) => records,
            Err(denial) => {
                self.unpublished_appearance = Err(denial);
                return self.appearance.reject_unpublished_output(Vec::new());
            }
        };
        if records.iter().any(|record| {
            matches!(
                record,
                crate::runtime::appearance::UiAppearanceInspectionRecord::Denial { denial, .. }
                    if denial.blocks_mounted_output()
            )
        }) {
            self.unpublished_appearance = Err(super::UiMountedAppearanceOutputDenial::NodeLowering);
            return self.appearance.reject_unpublished_output(records);
        }
        if let Err(_denial) =
            candidate.lower_retirements(self.projection.frame_identity(), presentation)
        {
            self.unpublished_appearance = Err(super::UiMountedAppearanceOutputDenial::NodeLowering);
            return self.appearance.reject_unpublished_output(records);
        }
        if let Err(denial) = candidate.admit_order(&self.projection) {
            self.unpublished_appearance =
                Err(super::UiMountedAppearanceOutputDenial::Order(denial));
            return self.appearance.reject_unpublished_output(records);
        }
        if let Err(_denial) =
            candidate.lower_overlays(&self.projection, presentation, &geometry, overlays)
        {
            self.unpublished_appearance = Err(super::UiMountedAppearanceOutputDenial::NodeLowering);
            return self.appearance.reject_unpublished_output(records);
        }
        match super::appearance_output::assemble(
            &self.projection,
            presentation,
            bindings,
            candidate.take_node_work(),
            candidate.take_overlay_work(),
            &self.pointer,
        ) {
            Ok((projection, pointers)) => {
                self.appearance = candidate;
                self.pointer = pointers;
                self.unpublished_appearance = Ok(projection.map(std::rc::Rc::new));
                records
            }
            Err(denial) => {
                self.unpublished_appearance = Err(denial);
                self.appearance.reject_unpublished_output(records)
            }
        }
    }

    pub(crate) fn deny_appearance_output(
        &mut self,
    ) -> Vec<crate::runtime::appearance::UiAppearanceInspectionRecord> {
        self.unpublished_appearance = Err(super::UiMountedAppearanceOutputDenial::NodeLowering);
        self.appearance.reject_unpublished_output(Vec::new())
    }
}
