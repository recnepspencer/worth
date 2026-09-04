use worth_ui_host_contract::{
    UiMountedFrameCanonicalCore, UiMountedFrameIntegrity, UiMountedFrameManifest,
    UiMountedSurfaceBindingRequirement,
};

use super::{
    UiMountedFramePreparationDenial, UiMountedFrameReceipt, UiMountedSurfaceReceipt,
    UiPreparedMountedFrame, UiPreparedMountedFrameAdmission,
};

impl UiPreparedMountedFrame {
    pub(crate) fn admit(
        admission: UiPreparedMountedFrameAdmission,
    ) -> Result<Self, UiMountedFramePreparationDenial> {
        let UiPreparedMountedFrameAdmission {
            candidate,
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
                    projection_frame: std::rc::Rc::clone(&candidate.frame),
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
        let cost = candidate.frame().cost_report();
        let identity_trace_basis = candidate.frame().identity_trace_basis(trace_source);
        Ok(Self {
            candidate,
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

    pub(crate) fn appearance_node_inputs(
        &self,
    ) -> Vec<super::super::projection::UiMountedAppearanceNodeInputContext> {
        self.candidate.frame().appearance_node_inputs()
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

    pub(crate) fn prune_appearance_state(
        &mut self,
        session: crate::facade::WorthUiActiveApplicationSessionIdentity,
        generation: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    ) {
        self.candidate.prune_appearance_state(session, generation);
    }

    pub(crate) fn reserve_appearance_state(
        &mut self,
        context: &crate::runtime::appearance::UiAppearanceAttemptContext,
    ) -> Result<(), super::super::projection::UiAppearanceStateCapacityExceeded> {
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
    ) -> Result<(), super::super::projection::UiAppearanceStateCapacityExceeded> {
        self.candidate.stage_appearance_projection(attempt)
    }

    pub(crate) fn lower_appearance(
        &mut self,
        presentation: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
    ) -> crate::runtime::appearance::UiAppearanceInspectionAttemptBatch {
        let invalidation = self.appearance_invalidation_batch();
        let records = self.candidate.lower_appearance(presentation);
        crate::runtime::appearance::UiAppearanceInspectionAttemptBatch::new(invalidation, records)
    }

    pub fn receipt(&self) -> UiMountedFrameReceipt {
        UiMountedFrameReceipt {
            canonical_core: self.canonical_core,
            integrity: self.integrity,
            surface_count: self.surfaces.len(),
            cost: self.cost,
        }
    }

    pub fn cost_report(&self) -> crate::mounting::UiMountCostReport {
        self.cost
    }

    pub fn reuse_contract(&self) -> &crate::mounting::UiMountedFrameReuseContract {
        &self.reuse_contract
    }

    pub(crate) fn visual_region_basis(&self) -> crate::mounting::UiMountedVisualRegionBasis {
        self.candidate.frame().visual_region_basis()
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

    pub(crate) fn focus_participation_snapshot(
        &self,
    ) -> crate::mounting::UiMountedFocusParticipationSnapshot {
        crate::mounting::UiMountedFocusParticipationSnapshot::from_projection(
            self.candidate.frame(),
            self.presented_receipt_basis(),
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
