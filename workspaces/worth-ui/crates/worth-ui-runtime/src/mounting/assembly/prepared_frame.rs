use worth_ui_host_contract::{
    UiMountedFrameCanonicalCore, UiMountedFrameIntegrity, UiMountedFrameManifest,
    UiMountedSurfaceBindingRequirement,
};

use super::{
    UiAssembledMountedFrame, UiMountedFramePreparationDenial, UiMountedFrameReceipt,
    UiMountedSurfaceReceipt, UiPreparedMountedFrameAdmission,
};

#[path = "prepared_frame/appearance_binding.rs"]
mod appearance_binding;

impl UiAssembledMountedFrame {
    pub(in crate::mounting) fn admit(
        admission: UiPreparedMountedFrameAdmission,
    ) -> Result<Self, UiMountedFramePreparationDenial> {
        let UiPreparedMountedFrameAdmission {
            mut candidate,
            text_publication,
            text_publication_work,
            generation,
            manifest,
            graph_world,
            allocation_truth_revision,
            trace_source,
            reuse_contract,
        } = admission;
        if trace_source.generation() != &generation {
            return Err(UiMountedFramePreparationDenial::TraceSourceGenerationMismatch);
        }
        crate::mounting::validate_manifest(&manifest)?;
        let surfaces = manifest
            .surfaces()
            .iter()
            .map(|requirement| {
                Ok(UiMountedSurfaceReceipt {
                    requirement: *requirement,
                    projection_frame: candidate.projection_rc(),
                    projection: std::cell::OnceCell::new(),
                })
            })
            .collect::<Result<Vec<_>, UiMountedFramePreparationDenial>>()?;
        let canonical_core = UiMountedFrameCanonicalCore::new(
            candidate.frame().frame_identity(),
            candidate.frame().plan_digest(),
            graph_world,
            allocation_truth_revision,
            candidate.frame().table_range_digest(),
        );
        let integrity = UiMountedFrameIntegrity::derive(canonical_core, &manifest);
        if !integrity.verifies(canonical_core, &manifest) {
            return Err(UiMountedFramePreparationDenial::IntegrityMismatch);
        }
        let mut cost = candidate.frame().cost_report();
        cost.record_text_publication_coverage_work(text_publication_work)
            .map_err(|_| {
                UiMountedFramePreparationDenial::Projection(
                    crate::mounting::UiMountedProjectionDenial::CostCounterOverflow,
                )
            })?;
        candidate.owner.application_text_publication = text_publication;
        let identity_trace_basis = candidate.frame().identity_trace_basis(trace_source);
        Ok(Self {
            candidate,
            appearance_retry_basis: None,
            prepared_theme_binding: None,
            generation,
            manifest,
            canonical_core,
            integrity,
            surfaces: surfaces.into_boxed_slice(),
            identity_trace_basis,
            cost,
            reuse_contract,
        })
    }

    pub fn generation(
        &self,
    ) -> &crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationIdentity
    {
        &self.generation
    }

    pub fn manifest(&self) -> &UiMountedFrameManifest {
        &self.manifest
    }

    pub fn canonical_core(&self) -> UiMountedFrameCanonicalCore {
        self.canonical_core
    }

    pub fn integrity(&self) -> UiMountedFrameIntegrity {
        self.integrity
    }

    pub fn surfaces(&self) -> &[UiMountedSurfaceReceipt] {
        &self.surfaces
    }

    pub(in crate::mounting) fn presentation_delta_source(
        &self,
    ) -> super::super::UiMountedPresentationDeltaSource<'_> {
        self.candidate.presentation_delta_source()
    }

    pub(in crate::mounting) fn semantic_projection(
        &self,
    ) -> &super::super::projection::UiMountedSemanticProjection {
        self.candidate.frame().semantic_projection()
    }

    pub(crate) fn appearance_attempt_inputs(
        &mut self,
        batch: &crate::runtime::appearance::UiAppearanceInvalidationBatch,
    ) -> Result<
        Vec<super::super::projection::UiMountedAppearanceNodeInputContext>,
        super::super::projection::UiMountedProjectionDenial,
    > {
        self.candidate.appearance_attempt_inputs(batch)
    }

    pub(crate) fn stage_appearance_input_refresh(
        &mut self,
        node: &super::super::projection::UiMountedAppearanceNodeInputContext,
    ) -> bool {
        self.candidate.stage_appearance_input_refresh(node)
    }

    pub(crate) fn projection_input(
        &self,
        slot: worth_ui_query_binding::UiProjectionInputSlot,
    ) -> Option<&worth_ui_query_binding::UiProjectionInputFactReference> {
        self.semantic_projection().projection_input(slot)
    }

    pub(crate) fn begin_appearance_lifecycle(
        &mut self,
        session: crate::facade::WorthUiActiveApplicationSessionIdentity,
        generation: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
        graph: crate::graph::UiGraphAuthority<'_>,
    ) -> Result<(), super::super::projection::UiMountedProjectionDenial> {
        self.validate_appearance_owner(session, generation)?;
        if !self.generation.matches_graph(graph) {
            return Err(super::super::projection::UiMountedProjectionDenial::AppearanceSelectionFrameMismatch);
        }
        self.candidate
            .begin_appearance_lifecycle(session, generation, graph);
        Ok(())
    }

    pub(crate) fn validate_appearance_owner(
        &self,
        session: crate::facade::WorthUiActiveApplicationSessionIdentity,
        generation: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    ) -> Result<(), super::super::projection::UiMountedProjectionDenial> {
        (generation.session_identity() == session
            && self.generation == *generation.prepared_generation())
        .then_some(())
        .ok_or(
            super::super::projection::UiMountedProjectionDenial::AppearanceSelectionFrameMismatch,
        )
    }

    pub(crate) fn validate_appearance_selection(
        &self,
        batch: &crate::runtime::appearance::UiAppearanceInvalidationBatch,
    ) -> Result<(), super::super::projection::UiMountedProjectionDenial> {
        self.candidate.validate_appearance_selection(batch)
    }

    pub(crate) fn appearance_selection_cost_report(
        &self,
    ) -> super::super::projection::UiMountedAppearanceSelectionCostReport {
        self.candidate.appearance_selection_cost_report()
    }

    pub(crate) fn appearance_motion_targets(
        &self,
        overlays: &[crate::mounting::UiMountedAppearanceSurfaceOverlayInput],
    ) -> Result<
        Vec<(
            worth_ui_host_contract::UiSemanticSurfaceIdentity,
            worth_ui_host_contract::UiMountedInstanceIdentity,
        )>,
        crate::mounting::projection::UiMountedAppearanceOutputDenial,
    > {
        self.candidate
            .appearance_motion_targets(self.manifest.surfaces(), overlays)
    }

    pub(crate) fn record_accepted_motion_commands_visited(&mut self, count: usize) {
        self.cost.record_appearance_motion_commands_visited(count);
    }

    #[cfg(test)]
    pub(crate) fn projection_rc_for_test(
        &self,
    ) -> std::rc::Rc<super::super::UiMountedProjectionFrame> {
        self.candidate.projection_rc()
    }

    pub(crate) fn set_appearance_invalidation_batch(
        &mut self,
        batch: crate::runtime::appearance::UiAppearanceInvalidationBatch,
    ) {
        self.candidate.set_appearance_invalidation_batch(batch);
    }

    pub(crate) fn appearance_invalidation_batch(
        &self,
    ) -> Option<crate::runtime::appearance::UiAppearanceInvalidationBatch> {
        self.candidate.appearance_invalidation_batch()
    }

    pub(crate) fn reserve_appearance_state(
        &mut self,
        context: &crate::runtime::appearance::UiAppearanceAttemptContext,
    ) -> Result<(), super::super::projection::UiMountedAppearanceStateMutationDenial> {
        self.candidate.reserve_appearance_state(context)
    }

    pub(crate) fn appearance_state_capacity_error(
        &self,
    ) -> Option<super::super::projection::UiAppearanceStateCapacityExceeded> {
        self.candidate.appearance_state_capacity_error()
    }

    pub(crate) fn stage_appearance_projection(
        &mut self,
        attempt: crate::runtime::appearance::UiAppearanceProjectionAttempt,
    ) -> Result<(), super::super::projection::UiMountedAppearanceStateMutationDenial> {
        self.candidate.stage_appearance_projection(attempt)
    }

    #[cfg(test)]
    pub(crate) fn lower_appearance(
        &mut self,
        presentation: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
        profile: Option<&worth_ui_host_contract::UiHostAppearanceProfileContract>,
    ) -> crate::runtime::appearance::UiAppearanceInspectionAttemptBatch {
        let invalidation = self.appearance_invalidation_batch();
        let records =
            self.candidate
                .lower_appearance(presentation, self.manifest.surfaces(), profile);
        crate::runtime::appearance::UiAppearanceInspectionAttemptBatch::new(invalidation, records)
    }

    pub(in crate::mounting::assembly) fn lower_appearance_with_motion_and_overlays(
        &mut self,
        presentation: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
        profile: Option<&worth_ui_host_contract::UiHostAppearanceProfileContract>,
        motion: crate::mounting::presentation::UiAcceptedAppearanceMotion,
        overlays: &[crate::mounting::UiMountedAppearanceSurfaceOverlayInput],
        scroll_chrome: &[crate::mounting::UiPresented<
            crate::mounting::UiMountedAppearanceScrollChromeInput,
        >],
    ) -> crate::runtime::appearance::UiAppearanceInspectionAttemptBatch {
        let invalidation = self.appearance_invalidation_batch();
        let records = self.candidate.lower_appearance_with_motion_and_overlays(
            presentation,
            self.manifest.surfaces(),
            profile,
            motion,
            overlays,
            scroll_chrome,
        );
        crate::runtime::appearance::UiAppearanceInspectionAttemptBatch::new(invalidation, records)
    }

    pub(crate) fn deny_appearance_output(
        &mut self,
    ) -> crate::runtime::appearance::UiAppearanceInspectionAttemptBatch {
        let invalidation = self.appearance_invalidation_batch();
        let records = self.candidate.deny_appearance_output();
        crate::runtime::appearance::UiAppearanceInspectionAttemptBatch::new(invalidation, records)
    }

    pub(crate) fn appearance_output_available(&self) -> bool {
        self.candidate.owner.unpublished_appearance().is_ok()
    }

    pub(crate) fn appearance_projection(
        &self,
    ) -> Option<&worth_ui_host_contract::UiUnpublishedAppearanceFrameProjection> {
        self.candidate
            .owner
            .unpublished_appearance()
            .expect("presentation begins only after appearance output admission")
    }

    pub fn receipt(&self) -> UiMountedFrameReceipt {
        UiMountedFrameReceipt {
            canonical_core: self.canonical_core,
            integrity: self.integrity,
            surface_count: self.surfaces.len(),
            cost: self.cost_report(),
        }
    }

    pub fn cost_report(&self) -> crate::mounting::UiMountCostReport {
        // Appearance and pointer admission finish after structural assembly.
        // Snapshot their completed measurements when issuing the receipt.
        self.cost.with_projection_work(
            self.candidate.appearance_selection_cost_report(),
            self.candidate
                .owner
                .projection()
                .hit_index_maintenance_work(),
        )
    }

    pub fn reuse_contract(&self) -> &crate::mounting::UiMountedFrameReuseContract {
        &self.reuse_contract
    }

    pub(in crate::mounting) fn diagnostic_source(
        &self,
    ) -> crate::mounting::projection::UiMountedDiagnosticSource {
        self.candidate.frame().diagnostic_source()
    }

    pub(crate) fn identity_trace_basis(&self) -> &crate::mounting::UiMountedIdentityTraceBasis {
        &self.identity_trace_basis
    }

    pub(crate) fn presented_receipt_basis(&self) -> &crate::mounting::UiMountedNodeReceiptBasis {
        self.candidate.presented_receipt_basis()
    }

    pub(in crate::mounting) fn focus_participation_snapshot(
        &self,
        mounted: &crate::mounting::UiMountedIdentityState,
    ) -> crate::mounting::UiMountedFocusParticipationSnapshot {
        let surfaces = self
            .manifest
            .surfaces()
            .iter()
            .map(|surface| surface.semantic_surface())
            .collect::<Vec<_>>();
        mounted.project_focus_participation(
            self.candidate.frame(),
            self.presented_receipt_basis(),
            &surfaces,
        )
    }

    pub(crate) fn into_publication_parts(
        self,
    ) -> (
        crate::mounting::UiProjectedMountedFrameCandidate,
        UiMountedFrameManifest,
        UiMountedFrameCanonicalCore,
        crate::mounting::UiMountedFrameReuseContract,
    ) {
        (
            self.candidate,
            self.manifest,
            self.canonical_core,
            self.reuse_contract,
        )
    }
}

pub(crate) fn binding_requirement(
    binding: crate::mounting::UiSurfaceBindingIdentityView,
) -> UiMountedSurfaceBindingRequirement {
    UiMountedSurfaceBindingRequirement::with_baseline_and_device_scale(
        binding.semantic_surface_identity(),
        binding.host_surface_identity(),
        binding.binding_generation(),
        binding.capability_observation_generation(),
        binding.capability_profile_digest(),
        binding.presentation_mode(),
        binding.baseline(),
        binding.profile().device_scale_milli(),
    )
}
