impl super::UiMountedGraphReplacementSuccessor {
    pub(crate) fn scroll_region_geometry(
        &self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        target: worth_ui_host_contract::UiMountedInstanceIdentity,
        slot: usize,
    ) -> Option<(
        worth_ui_host_contract::UiMountedInstanceIdentity,
        worth_ui_host_contract::UiMountedCanonicalBox,
        worth_ui_host_contract::UiMountedCanonicalBox,
    )> {
        self.occurrence_geometry
            .scroll_region_geometry(surface, target, slot)
    }

    pub(crate) fn layout_validation_identity(&self) -> &crate::mounting::UiMountedIdentityState {
        &self.identity
    }

    pub(crate) fn candidate_layout_basis(
        &self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        generation: crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationIdentity,
    ) -> Result<
        crate::mounting::UiMountedLayoutBasis,
        crate::mounting::UiMountedOccurrenceGeometryDenial,
    > {
        self.identity.layout_basis(surface, generation)
    }

    pub(crate) fn next_candidate_layout_revision(
        &self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) -> Result<
        crate::mounting::UiMountedLayoutRevision,
        crate::mounting::UiMountedOccurrenceGeometryDenial,
    > {
        self.occurrence_geometry.next_layout_revision(surface)
    }

    pub(crate) fn candidate_layout_viewport(
        &self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) -> Option<worth_ui_host_contract::UiMountedCanonicalBox> {
        self.occurrence_geometry
            .surface_viewport(surface)
            .map(|(_, _, viewport)| viewport)
    }

    pub(crate) fn replace_candidate_occurrence_geometry(
        &mut self,
        batch: crate::mounting::UiMountedSurfaceGeometryBatch,
        scroll: Option<&mut crate::runtime::scroll::UiScrollRuntimeState>,
    ) -> Result<
        crate::mounting::UiMountedLayoutCompletionReceipt,
        crate::mounting::UiMountedOccurrenceGeometryDenial,
    > {
        let surface = batch.surface();
        let revision = batch.layout_revision();
        let binding = self
            .identity
            .projection_surface(surface)
            .ok_or(crate::mounting::UiMountedOccurrenceGeometryDenial::MissingSurfaceBinding)?
            .0
            .binding_generation();
        let graph_world = self.identity.world_identity();
        let (changed, region_rows, region_lookups, seam_rows, seam_adjacencies) = self
            .occurrence_geometry
            .replace_surface(&self.identity, batch)?;
        let mut changed = changed.into_vec();
        if let Some(scroll) = scroll {
            changed.extend(
                self.occurrence_geometry
                    .restore_scroll_geometry(surface, &self.identity, scroll)?
                    .into_vec(),
            );
        }
        changed.sort_unstable();
        changed.dedup();
        if !changed.is_empty() {
            self.identity
                .mark_occurrence_geometry_changed(&changed)
                .map_err(|_| {
                    crate::mounting::UiMountedOccurrenceGeometryDenial::UnknownMountedInstance
                })?;
        }
        Ok(crate::mounting::UiMountedLayoutCompletionReceipt::new(
            surface,
            binding,
            graph_world,
            revision,
            changed.len(),
        )
        .add_region_resolution_work(region_rows, region_lookups)
        .with_seam_resolution_work(seam_rows, seam_adjacencies))
    }

    pub(crate) fn prepare_occurrence_geometry_succession(
        &mut self,
        previous: &crate::runtime::session::WorthUiApplicationSessionState,
        candidate: &crate::facade::prepared_application_authority::WorthUiPreparedApplicationAuthority,
        plan: &crate::runtime::WorthUiActiveExecutionPlan,
        bindings: &crate::runtime::portal::UiPortalOverlayBindingLifecycle,
    ) -> Result<(), crate::mounting::UiMountedOccurrenceGeometryDenial> {
        self.occurrence_geometry.prepare_generation_succession(
            &self.identity,
            previous,
            candidate,
            plan,
            bindings,
        )
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
        Some(
            self.occurrence_geometry
                .region_extents(surface, generation, region)?
                .iter()
                .map(|(owner, _, bounds)| (*owner, *bounds))
                .collect(),
        )
    }
}
