use super::WorthUiMountedSessionState;

pub(crate) struct UiMountedPaintAttribution {
    pub(crate) surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    pub(crate) mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    pub(crate) node_receipt: worth_ui_host_contract::UiMountedNodeReceiptIdentity,
    pub(crate) authored_provenance_digest: u64,
    pub(crate) authored_semantic_identity_digest: u64,
}

impl WorthUiMountedSessionState {
    pub(crate) fn projection_changes_pending(&self) -> bool {
        self.identity.projection_changes_pending()
    }

    pub(crate) fn admits_retained_appearance_generation(
        &self,
        owners: &crate::runtime::appearance::UiPreparedRetainedAppearanceOwnerSuccession,
    ) -> bool {
        self.identity.admits_retained_appearance_generation(owners)
    }

    pub(crate) fn commit_retained_appearance_generation(
        &mut self,
        owners: &crate::runtime::appearance::UiPreparedRetainedAppearanceOwnerSuccession,
    ) {
        self.identity.commit_retained_appearance_generation(owners);
    }
    pub(crate) fn layout_basis(
        &self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        generation: crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationIdentity,
    ) -> Result<
        crate::mounting::UiMountedLayoutBasis,
        crate::mounting::UiMountedOccurrenceGeometryDenial,
    > {
        self.identity.layout_basis(surface, generation)
    }

    pub(crate) fn current_text_publication_for_frame(
        &self,
        frame: worth_ui_host_contract::UiMountedFrameIdentity,
    ) -> Option<&crate::runtime::presentation_state::UiApplicationTextPublication> {
        let owner = self.identity.current_projection_owner()?;
        (owner.projection().frame_identity() == frame)
            .then_some(owner.application_text_publication.as_deref())
            .flatten()
    }

    #[cfg(test)]
    pub(crate) fn current_pointer_affordance_for_test(
        &self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) -> Option<worth_ui_host_contract::UiMountedPointerAffordanceMechanic> {
        self.identity
            .pointer_predecessor()?
            .retained_mechanic_for_test(surface)
    }

    pub(crate) fn pointer_presentation_pending(
        &self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        binding: worth_ui_host_contract::UiSurfaceBindingGeneration,
        snapshot: Option<&crate::runtime::pointer_affordance::UiPointerAffordanceSnapshot>,
    ) -> bool {
        self.identity.pointer_predecessor().map_or_else(
            || {
                snapshot.is_some_and(|snapshot| {
                    snapshot
                        .active_projections()
                        .any(|row| row.surface() == surface && row.target().is_some())
                })
            },
            |committed| !committed.matches_snapshot(surface, binding, snapshot),
        )
    }

    #[cfg(test)]
    pub(crate) fn current_unpublished_appearance(
        &self,
    ) -> Result<
        Option<&worth_ui_host_contract::UiUnpublishedAppearanceFrameProjection>,
        &crate::mounting::projection::UiMountedAppearanceOutputDenial,
    > {
        self.identity
            .current_projection_owner()
            .map_or(Ok(None), |owner| owner.unpublished_appearance())
    }

    #[cfg(test)]
    pub(crate) fn current_projection_rc_for_test(
        &self,
    ) -> Option<std::rc::Rc<crate::mounting::UiMountedProjectionFrame>> {
        self.identity
            .current_projection_owner()
            .map(|owner| owner.projection_rc())
    }

    pub(crate) fn seal_appearance_receipt_basis(
        &self,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
        incarnation: worth_ui_host_contract::UiMountIncarnation,
        successor: &crate::mounting::UiMountedNodeReceiptBasis,
    ) -> Result<
        crate::mounting::UiMountedAppearanceReceiptBasis,
        crate::mounting::UiMountedAppearanceReceiptBasisDenial,
    > {
        self.identity
            .seal_appearance_receipt_basis(instance, incarnation, successor)
    }

    pub(crate) fn current_mounted_identity_basis(
        &self,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) -> Option<crate::mounting::UiMountedIdentityBasis> {
        self.identity
            .projection_instance(instance)
            .map(|view| view.basis().clone())
    }

    #[cfg(test)]
    pub(crate) fn current_region_extent(
        &self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        generation: &crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationIdentity,
        region: worth_ui_dsl::UiMosaicRegionDeclarationIdentity,
    ) -> Option<worth_ui_host_contract::UiMountedCanonicalBox> {
        match self
            .current_region_extents(surface, generation, region)?
            .as_ref()
        {
            [(_, bounds)] => Some(*bounds),
            _ => None,
        }
    }

    pub(crate) fn current_region_extents(
        &self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        generation: &crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationIdentity,
        region: worth_ui_dsl::UiMosaicRegionDeclarationIdentity,
    ) -> Option<
        Box<
            [(
                worth_ui_host_contract::UiMountedInstanceIdentity,
                worth_ui_host_contract::UiMountedCanonicalBox,
            )],
        >,
    > {
        self.current_surface_viewport(surface)?;
        let rows = self
            .occurrence_geometry
            .region_extents(surface, generation, region)?;
        Some(
            rows.iter()
                .filter_map(|(instance, incarnation, bounds)| {
                    let view = self.identity.projection_instance(*instance)?;
                    (view.mount_incarnation() == *incarnation
                        && view.basis().semantic_surface_identity() == surface)
                        .then_some((*instance, *bounds))
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        )
    }

    pub(crate) fn validate_appearance_receipt_basis(
        &self,
        basis: crate::mounting::UiMountedAppearanceReceiptBasis,
    ) -> Result<(), crate::mounting::UiMountedAppearanceReceiptBasisDenial> {
        self.identity.validate_appearance_receipt_basis(basis)
    }

    pub(crate) fn current_surface_for_binding(
        &self,
        binding: worth_ui_host_contract::UiSurfaceBindingGeneration,
    ) -> Option<worth_ui_host_contract::UiSemanticSurfaceIdentity> {
        self.identity
            .current_projection()?
            .view_for(binding)
            .ok()
            .map(|view| view.surface())
    }

    pub(crate) fn current_surfaces(
        &self,
    ) -> impl Iterator<Item = worth_ui_host_contract::UiSemanticSurfaceIdentity> + '_ {
        self.identity.bound_surfaces()
    }

    pub(crate) fn current_portal_owner_for_child(
        &self,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) -> Option<(
        crate::graph::UiGraphNodeIdentity,
        worth_ui_host_contract::UiMountedInstanceIdentity,
    )> {
        self.identity
            .current_projection()?
            .portal_owner_for_child(instance)
    }

    pub(crate) fn current_portal_content_extent(
        &self,
        owner: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) -> Result<
        Option<crate::runtime::portal::UiPortalContentBounds>,
        crate::mounting::UiMountedProjectionDenial,
    > {
        self.identity
            .current_projection()
            .ok_or(crate::mounting::UiMountedProjectionDenial::PortalOverlayOwnerMissing)?
            .portal_content_extent(owner)
    }

    pub(crate) fn native_observed_paint_attribution(
        &self,
        frame: worth_ui_host_contract::UiMountedFrameIdentity,
        binding: worth_ui_host_contract::UiSurfaceBindingGeneration,
        observed_surface: u64,
        observed_instance: u64,
        observed_receipt: u64,
    ) -> Option<UiMountedPaintAttribution> {
        let view = self.identity.current_projection()?.view_for(binding).ok()?;
        if view.frame() != frame || view.surface().diagnostic_value() != observed_surface {
            return None;
        }
        // Host observations identify what was drawn; only accepted mounting can
        // attach authored provenance. No paint ordering is reconstructed here.
        let node = view.nodes().iter().find(|node| {
            node.mounted_instance().diagnostic_value() == observed_instance
                && node.node_receipt().diagnostic_value() == observed_receipt
        })?;
        let authored = self
            .identity
            .current_authored_attribution(node.mounted_instance())?;
        Some(UiMountedPaintAttribution {
            surface: view.surface(),
            mounted_instance: node.mounted_instance(),
            node_receipt: node.node_receipt(),
            authored_provenance_digest: authored.source_provenance_digest,
            authored_semantic_identity_digest: authored.semantic_identity_digest,
        })
    }

    pub(crate) fn classify_frame_reuse(
        &self,
        contract: crate::mounting::UiMountedFrameReuseContract,
    ) -> crate::mounting::UiMountedFrameReuse {
        self.identity.classify_reuse(contract)
    }

    pub(crate) fn pointer_projection_is_current(
        &self,
        snapshot: &crate::runtime::pointer_affordance::UiPointerAffordanceSnapshot,
        projection: &crate::runtime::pointer_affordance::UiPointerAffordanceProjection,
    ) -> bool {
        if self
            .identity
            .projection_surface(projection.surface())
            .is_none()
        {
            return false;
        }
        if self
            .identity
            .pointer_predecessor()
            .is_some_and(|state| state.has_admitted_observation(projection.surface(), snapshot))
        {
            return true;
        }
        projection.presented_target().is_none_or(|target| {
            self.current_semantic_surface_for_presentation(target.presentation())
                == Ok(projection.surface())
                && self
                    .validate_current_receipt(target.mounted_instance(), target.node_receipt())
                    .is_ok()
        })
    }

    pub(crate) fn admit_pointer_reuse_observation(
        &mut self,
        snapshot: Option<&crate::runtime::pointer_affordance::UiPointerAffordanceSnapshot>,
        request: &crate::mounting::UiMountedFrameRequest,
    ) -> Result<(), crate::mounting::UiMountedFramePreparationDenial> {
        let Some(snapshot) = snapshot else {
            return Ok(());
        };
        let predecessor = self.identity.pointer_predecessor();
        let mut admitted_count = 0;
        let mut already_admitted = true;
        for projection in snapshot.active_projections() {
            if self.pointer_projection_is_current(snapshot, projection) {
                admitted_count += 1;
                already_admitted &= predecessor.is_some_and(|state| {
                    state.has_admitted_observation(projection.surface(), snapshot)
                });
            } else if request.includes_surface(projection.surface()) {
                return Err(match projection.target() {
                    Some(target) => crate::mounting::UiMountedFramePreparationDenial::PointerSnapshotTargetUnavailable(target),
                    None => crate::mounting::UiMountedFramePreparationDenial::MissingSurfaceBinding(projection.surface()),
                });
            }
        }
        // Reuse re-admits the observation the owner already holds; rebuilding
        // an identical admission would put allocation on the unchanged path.
        if already_admitted
            && predecessor.map_or(0, |state| state.admitted_surface_count()) == admitted_count
        {
            return Ok(());
        }
        let admitted = snapshot
            .active_projections()
            .filter(|projection| self.pointer_projection_is_current(snapshot, projection))
            .map(|projection| projection.surface())
            .collect::<Vec<_>>();
        self.identity
            .retain_pointer_observation_admission(snapshot.observation_identity(), &admitted);
        Ok(())
    }

    pub(crate) fn current_allocation_truth_revision(&self) -> Option<u64> {
        self.identity.current_allocation_truth_revision()
    }

    pub(crate) fn seal_frame_reuse_contract(
        &self,
        basis: crate::mounting::UiMountedFrameReuseExternalBasis,
    ) -> crate::mounting::UiMountedFrameReuseContract {
        self.identity.seal_reuse_contract(basis)
    }

    pub(crate) fn begin_frame_assembly(
        &self,
        input: crate::mounting::UiMountedFrameAssemblyInput<'_, '_>,
    ) -> Result<
        crate::mounting::UiMountedFrameAssembler<'_>,
        crate::mounting::UiMountedFramePreparationDenial,
    > {
        crate::mounting::UiMountedFrameAssembler::begin(
            &self.identity,
            &self.occurrence_geometry,
            input,
        )
    }

    pub(crate) fn begin_frame_assembly_with_occurrence_geometry<'state>(
        &'state self,
        occurrence_geometry: &'state crate::mounting::UiMountedOccurrenceGeometryState,
        input: crate::mounting::UiMountedFrameAssemblyInput<'_, '_>,
    ) -> Result<
        crate::mounting::UiMountedFrameAssembler<'state>,
        crate::mounting::UiMountedFramePreparationDenial,
    > {
        crate::mounting::UiMountedFrameAssembler::begin(&self.identity, occurrence_geometry, input)
    }

    pub(crate) fn begin_superseding_frame_assembly<'state>(
        &'state self,
        predecessor: &'state crate::mounting::UiPreparedMountedFrame,
        input: crate::mounting::UiMountedFrameAssemblyInput<'_, '_>,
    ) -> Result<
        crate::mounting::UiMountedFrameAssembler<'state>,
        crate::mounting::UiMountedFramePreparationDenial,
    > {
        crate::mounting::UiMountedFrameAssembler::begin_graph_replacement(
            &self.identity,
            &self.occurrence_geometry,
            Some(predecessor.semantic_projection()),
            Some(predecessor.canonical_core().frame()),
            input,
        )
    }

    pub(crate) fn current_projection_input(
        &self,
        slot: worth_ui_query_binding::UiProjectionInputSlot,
    ) -> Option<worth_ui_query_binding::UiProjectionInputFactReference> {
        self.retention.current_projection_input(slot)
    }
}
