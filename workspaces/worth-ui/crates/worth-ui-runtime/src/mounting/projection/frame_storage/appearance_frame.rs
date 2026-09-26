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
        Ok(node.has_appearance_attachment && node.portal_surface_appearance)
    }

    /// Whether the occurrence paints an appearance of its own this frame. An
    /// occurrence that does not can still own scroll chrome, which is then
    /// lowered as a fragment of its own rather than inside a node fragment.
    pub(super) fn appearance_attached(
        &self,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) -> Result<bool, super::super::UiMountedProjectionDenial> {
        self.semantic
            .node(instance)
            .map(|node| node.has_appearance_attachment)
            .ok_or(super::super::UiMountedProjectionDenial::AppearanceSelectionFrameMismatch)
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
        let appearance_geometry = node.completed_appearance_geometry();
        Ok(super::UiMountedAppearanceNodeInputContext {
            frame: self.frame,
            semantic_surface: receipt.semantic_surface(),
            mounted_instance: instance,
            graph_node: receipt.graph_node(),
            incarnation: receipt.incarnation(),
            node_receipt,
            issuer: self.receipt_basis.issuer(),
            plan_digest: receipt.plan_digest(),
            allocation: appearance_geometry.allocation().into_shown(),
            appearance_clip: appearance_geometry.clip,
            surface_paint_order: node.surface_paint_order,
            portal_group: appearance_geometry.portal_group(),
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

    pub(super) fn appearance_portal_surface_input(
        &self,
        portal: worth_ui_host_contract::UiMountedPortalOverlayMechanic,
    ) -> Result<super::UiMountedAppearanceNodeInputContext, super::super::UiMountedProjectionDenial>
    {
        use worth_ui_host_contract::UiMountedAllocationProjection as Allocation;
        let mut context = self.appearance_node_input(portal.owner())?;
        if portal.frame() != self.frame
            || portal.surface() != context.semantic_surface
            || portal.owner_receipt() != context.node_receipt
            || self
                .semantic
                .surface_for(context.semantic_surface)
                .map(|surface| surface.binding)
                != Some(portal.binding())
        {
            return Err(super::super::UiMountedProjectionDenial::PortalOverlayOwnerMissing);
        }
        let basis = match context.allocation {
            Allocation::Known { basis, .. } | Allocation::PortalAnchorObservation { basis, .. } => {
                basis
            }
            Allocation::Omitted(_) => {
                return Err(super::super::UiMountedProjectionDenial::PortalOverlayOwnerMissing);
            }
        };
        // The anchor retains its ordinary surface. The separate Portal surface
        // consumes the Portal owner's presented rectangle and clipping authority.
        context.allocation = Allocation::Known {
            bounds: portal.bounds(),
            basis,
        };
        context.appearance_clip = super::super::appearance::portal_surface_clip(portal)
            .map_err(|_| super::super::UiMountedProjectionDenial::NonFiniteGeometry)?;
        context.portal_group = Some(portal.owner());
        // Regional seams and text belong to the anchor occurrence, not this
        // independently placed surface. Overlay retention owns its geometry.
        context.geometry_input = None;
        context.text_foreground_spans = Box::new([]);
        Ok(context)
    }
}

impl UiMountedProjectionFrameOwner {
    pub(crate) fn admits_retained_appearance_generation(
        &self,
        owners: &crate::runtime::appearance::UiPreparedRetainedAppearanceOwnerSuccession,
    ) -> bool {
        self.appearance.admits_retained_generation(owners)
    }

    pub(crate) fn commit_retained_appearance_generation(
        &mut self,
        owners: &crate::runtime::appearance::UiPreparedRetainedAppearanceOwnerSuccession,
    ) {
        self.appearance.commit_retained_generation(owners);
    }
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
        self.appearance
            .selection_cost_report()
            .with_pointer_work(self.pointer.work())
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
        let mut staged_appearance = None;
        if !overlays.is_empty()
            || self.appearance.has_active_portal_instances()
            || !self.projection.portal_overlay_inputs().is_empty()
        {
            let mut candidate = self.appearance.clone();
            candidate.stage_portal_ownership_changes(&self.projection, bindings, overlays)?;
            staged_appearance = Some(candidate);
        }
        let appearance = staged_appearance.as_ref().unwrap_or(&self.appearance);
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
        // Reconstructed and refreshed retained roles are lowered without a new
        // resolver selection. Their actual prepared inputs still consume Motion.
        targets.extend(appearance.refreshed_motion_targets());
        targets.extend(overlays.iter().flat_map(|overlay| {
            overlay
                .backdrops
                .iter()
                .filter_map(|backdrop| backdrop.motion_target)
                .map(|target| (overlay.semantic_surface, target.owner()))
        }));
        // Overlay lowering reconstructs every active Portal surface, including
        // unchanged children with no Backdrop Motion dependency. Its accepted
        // command samples remain required even when no role slot was selected.
        targets.extend(overlays.iter().flat_map(|overlay| {
            overlay
                .portal_instances
                .iter()
                .map(|instance| (overlay.semantic_surface, *instance))
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
}
