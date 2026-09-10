use super::WorthUiMountedSessionState;
use crate::mounting::{
    UiHostSurfaceIdentity, UiHostSurfacePresentationMode, UiMountedFrameIdentity,
    UiMountedGraphNodeHandle, UiMountedIdentityDenial, UiMountedIdentityView,
    UiMountedInstanceIdentity, UiMountedNodeReceiptIdentity, UiMountedProjectionAudience,
    UiSemanticSurfaceIdentity, UiSurfaceBindingGeneration, UiSurfaceBindingIdentityView,
    UiSurfaceBindingProfile,
};

impl WorthUiMountedSessionState {
    pub(crate) fn create_semantic_surface(
        &mut self,
    ) -> Result<UiSemanticSurfaceIdentity, UiMountedIdentityDenial> {
        self.ensure_identity_mutation_available()?;
        self.identity.create_semantic_surface()
    }

    pub(crate) fn create_semantic_surface_for(
        &mut self,
        audience: UiMountedProjectionAudience,
    ) -> Result<UiSemanticSurfaceIdentity, UiMountedIdentityDenial> {
        self.ensure_identity_mutation_available()?;
        self.identity.create_semantic_surface_for(audience)
    }

    pub(crate) fn graph_node_handle(
        &self,
        graph: crate::graph::UiGraphAuthority<'_>,
        graph_node_identity: crate::graph::UiGraphNodeIdentity,
    ) -> Result<UiMountedGraphNodeHandle, UiMountedIdentityDenial> {
        self.identity.graph_node_handle(graph, graph_node_identity)
    }

    pub(crate) fn register_host_surface(
        &mut self,
        host: &crate::facade::WorthUiHostSessionAuthority,
        semantic_surface: UiSemanticSurfaceIdentity,
        mode: UiHostSurfacePresentationMode,
        profile: UiSurfaceBindingProfile,
    ) -> Result<UiSurfaceBindingIdentityView, UiMountedIdentityDenial> {
        self.ensure_identity_mutation_available()?;
        let candidate = self.identity.prepare_surface_registration(
            host.protocol(),
            host.capability_report(),
            semantic_surface,
            mode,
            profile,
        )?;
        let baseline = self
            .presentation
            .host_truth_mut()
            .register_surface(host.effect_port(), candidate.request())?;
        Ok(self
            .identity
            .commit_surface_registration(candidate, baseline))
    }

    pub(crate) fn deregister_host_surface(
        &mut self,
        host: &crate::facade::WorthUiHostSessionAuthority,
        binding: UiSurfaceBindingGeneration,
    ) -> Result<UiSemanticSurfaceIdentity, UiMountedIdentityDenial> {
        self.deregister_host_surface_with_retention(host, binding, false)
    }

    pub(crate) fn deregister_host_surface_for_rebind(
        &mut self,
        host: &crate::facade::WorthUiHostSessionAuthority,
        binding: UiSurfaceBindingGeneration,
    ) -> Result<(UiSemanticSurfaceIdentity, UiHostSurfaceIdentity), UiMountedIdentityDenial> {
        let host_surface = self
            .identity
            .surface_binding(binding)
            .ok_or(UiMountedIdentityDenial::UnknownSurfaceBinding)?
            .host_surface_identity();
        self.deregister_host_surface_with_retention(host, binding, true)
            .map(|semantic| (semantic, host_surface))
    }

    pub(crate) fn register_rebound_host_surface(
        &mut self,
        host: &crate::facade::WorthUiHostSessionAuthority,
        prior_binding: UiSurfaceBindingGeneration,
        semantic_surface: UiSemanticSurfaceIdentity,
        host_surface: UiHostSurfaceIdentity,
        mode: UiHostSurfacePresentationMode,
        profile: UiSurfaceBindingProfile,
    ) -> Result<UiSurfaceBindingIdentityView, UiMountedIdentityDenial> {
        self.ensure_identity_mutation_available()?;
        let candidate = match self.identity.prepare_surface_rebind_registration(
            host.protocol(),
            host.capability_report(),
            semantic_surface,
            host_surface,
            mode,
            profile,
        ) {
            Ok(candidate) => candidate,
            Err(denial) => {
                self.presentation.abandon_surface_rebind(prior_binding);
                return Err(denial);
            }
        };
        let baseline = match self
            .presentation
            .host_truth_mut()
            .register_surface(host.effect_port(), candidate.request())
        {
            Ok(baseline) => baseline,
            Err(denial) => {
                self.presentation.abandon_surface_rebind(prior_binding);
                return Err(denial);
            }
        };
        let view = self
            .identity
            .commit_surface_registration(candidate, baseline);
        self.occurrence_geometry
            .rebind_surface(semantic_surface, view.binding_generation());
        self.presentation.commit_surface_rebind(prior_binding, view);
        Ok(view)
    }

    fn deregister_host_surface_with_retention(
        &mut self,
        host: &crate::facade::WorthUiHostSessionAuthority,
        binding: UiSurfaceBindingGeneration,
        preserve_for_rebind: bool,
    ) -> Result<UiSemanticSurfaceIdentity, UiMountedIdentityDenial> {
        self.ensure_identity_mutation_available()?;
        let requires_reconciliation = self.presentation.binding_requires_reconciliation(binding);
        let required_by_current = self.identity.current_requires_binding(binding);
        let current_requirement = self.identity.current_binding_requirement(binding);
        let has_published_predecessor = self.identity.publication_receipt().is_some();
        let preserve_published_frame = has_published_predecessor
            && (preserve_for_rebind || requires_reconciliation || !required_by_current);
        let candidate = self
            .identity
            .prepare_surface_deregistration(binding, preserve_published_frame)?;
        let text_pin_candidate = self.presentation.prepare_text_pin_deregistration(binding);
        self.presentation
            .host_truth_mut()
            .deregister_surface(host.effect_port(), candidate.request())?;
        if preserve_for_rebind && has_published_predecessor && required_by_current {
            self.presentation.host_truth_mut().block_presentation(
                current_requirement.expect("a current binding has its exact surface requirement"),
            );
        }
        self.presentation.commit_surface_deregistration(
            binding,
            text_pin_candidate,
            preserve_for_rebind,
        );
        let semantic_surface = self.identity.commit_surface_deregistration(candidate);
        if !preserve_for_rebind {
            self.selection_bindings.retire_surface(semantic_surface);
            self.occurrence_geometry.retire_surface(semantic_surface);
        }
        if has_published_predecessor && requires_reconciliation && !required_by_current {
            self.presentation
                .reconcile_candidate_only_deregistration(binding);
        }
        Ok(semantic_surface)
    }

    pub(crate) fn recover_indeterminate_host_surface(
        &mut self,
        host: &crate::facade::WorthUiHostSessionAuthority,
        semantic_surface: UiSemanticSurfaceIdentity,
    ) -> Result<(), UiMountedIdentityDenial> {
        self.ensure_identity_mutation_available()?;
        self.presentation
            .host_truth_mut()
            .recover_surface_effect(host.effect_port(), semantic_surface)
    }

    pub(crate) fn mount_instance(
        &mut self,
        graph: crate::graph::UiGraphAuthority<'_>,
        node: UiMountedGraphNodeHandle,
        surface: UiSemanticSurfaceIdentity,
    ) -> Result<UiMountedInstanceIdentity, UiMountedIdentityDenial> {
        self.ensure_identity_mutation_available()?;
        self.identity.mount(graph, node, surface)
    }

    pub(crate) fn replace_occurrence_geometry(
        &mut self,
        batch: super::super::UiMountedSurfaceGeometryBatch,
    ) -> Result<
        super::super::UiMountedLayoutCompletionReceipt,
        super::super::UiMountedOccurrenceGeometryDenial,
    > {
        if self.has_active_presentation_attempt() {
            return Err(super::super::UiMountedOccurrenceGeometryDenial::PresentationInFlight);
        }
        // Validate and build the complete surface replacement away from the
        // live table. Revision exhaustion must not leave geometry advanced
        // without the semantic currentness change that revokes prepared work.
        let surface = batch.surface();
        let revision = batch.layout_revision();
        let binding = self
            .identity
            .projection_surface(surface)
            .ok_or(super::super::UiMountedOccurrenceGeometryDenial::MissingSurfaceBinding)?
            .0
            .binding_generation();
        let graph_world = self.identity.world_identity();
        let mut successor = self.occurrence_geometry.clone();
        let (
            changed,
            region_index_rows,
            region_lookup_steps,
            seam_index_rows,
            seam_adjacencies_visited,
        ) = successor.replace_surface(&self.identity, batch)?;
        if !changed.is_empty() {
            self.identity
                .mark_occurrence_geometry_changed(&changed)
                .map_err(|denial| match denial {
                    super::super::UiMountedIdentityDenial::UnknownMountedInstance => {
                        super::super::UiMountedOccurrenceGeometryDenial::UnknownMountedInstance
                    }
                    super::super::UiMountedIdentityDenial::IdentityExhausted => {
                        super::super::UiMountedOccurrenceGeometryDenial::StateRevisionExhausted
                    }
                    _ => {
                        unreachable!(
                            "occurrence change validates membership before revision minting"
                        )
                    }
                })?;
        }
        self.occurrence_geometry = successor;
        Ok(super::super::UiMountedLayoutCompletionReceipt::new(
            surface,
            binding,
            graph_world,
            revision,
            changed.len(),
        )
        .add_region_resolution_work(region_index_rows, region_lookup_steps)
        .with_seam_resolution_work(seam_index_rows, seam_adjacencies_visited))
    }

    pub(crate) fn validate_occurrence_geometry_batch(
        &self,
        batch: &super::super::UiMountedSurfaceGeometryBatch,
    ) -> Result<(), super::super::UiMountedOccurrenceGeometryDenial> {
        if self.has_active_presentation_attempt() {
            return Err(super::super::UiMountedOccurrenceGeometryDenial::PresentationInFlight);
        }
        let mut candidate = self.occurrence_geometry.clone();
        candidate
            .replace_surface(&self.identity, batch.clone())
            .map(drop)
    }

    pub(crate) fn next_occurrence_geometry_revision(
        &self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) -> Result<
        super::super::UiMountedLayoutRevision,
        super::super::UiMountedOccurrenceGeometryDenial,
    > {
        self.occurrence_geometry.next_layout_revision(surface)
    }

    #[cfg(test)]
    pub(crate) fn next_occurrence_geometry_revision_for_test(
        &self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) -> super::super::UiMountedLayoutRevision {
        self.next_occurrence_geometry_revision(surface)
            .expect("test layout revision remains available")
    }

    pub(crate) fn unmount_instance(
        &mut self,
        identity: UiMountedInstanceIdentity,
    ) -> Result<(), UiMountedIdentityDenial> {
        self.ensure_identity_mutation_available()?;
        let mut successor_geometry = self.occurrence_geometry.clone();
        let affected = successor_geometry.retire_instance(identity);
        self.identity.unmount(identity, &affected)?;
        self.occurrence_geometry = successor_geometry;
        self.selection_bindings.retire_mount(identity);
        Ok(())
    }

    pub(crate) fn reorder_mounted_instances(
        &mut self,
        order: &[UiMountedInstanceIdentity],
    ) -> Result<(), UiMountedIdentityDenial> {
        self.ensure_identity_mutation_available()?;
        self.identity.reorder(order)
    }

    pub(crate) fn mounted_instances_for(
        &self,
        node: UiMountedGraphNodeHandle,
    ) -> Result<Box<[UiMountedInstanceIdentity]>, UiMountedIdentityDenial> {
        self.identity.instances_for(node)
    }

    pub(crate) fn advance_frame(
        &mut self,
    ) -> Result<UiMountedFrameIdentity, UiMountedIdentityDenial> {
        self.ensure_identity_mutation_available()?;
        self.identity.advance_frame()
    }

    pub(crate) fn validate_binding(
        &self,
        binding: UiSurfaceBindingGeneration,
    ) -> Result<(), UiMountedIdentityDenial> {
        self.identity.validate_binding(binding)
    }

    pub(crate) fn validate_current_frame(
        &self,
        frame: UiMountedFrameIdentity,
    ) -> Result<(), UiMountedIdentityDenial> {
        self.identity.validate_current_frame(frame)
    }

    pub(crate) fn validate_current_receipt(
        &self,
        instance: UiMountedInstanceIdentity,
        receipt: UiMountedNodeReceiptIdentity,
    ) -> Result<(), UiMountedIdentityDenial> {
        self.identity.validate_current_receipt(instance, receipt)
    }

    pub(crate) fn view(&self) -> UiMountedIdentityView {
        self.identity.view()
    }

    pub(crate) fn focus_participation_snapshot(
        &self,
    ) -> Option<crate::mounting::UiMountedFocusParticipationSnapshot> {
        self.identity.focus_participation_snapshot()
    }

    fn ensure_identity_mutation_available(&self) -> Result<(), UiMountedIdentityDenial> {
        if self.has_active_presentation_attempt() {
            return Err(UiMountedIdentityDenial::PresentationInFlight);
        }
        Ok(())
    }
}
