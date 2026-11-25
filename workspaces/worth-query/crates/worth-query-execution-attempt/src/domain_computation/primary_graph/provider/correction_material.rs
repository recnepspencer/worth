//! Current receiver attachment and original-source joins for correction material.
use super::WorthQueryPrimaryGraphProvider;
use crate::domain_computation::application_aftermath::correction_material::{
    QueryCorrectionMaterialDenial as Denial, QueryCorrectionsMedium,
};
use crate::domain_computation::primary_graph::retained_correction::{
    catalog::{MaterialBody, MaterialCatalog},
    CorrectionMaterialOwner,
};
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_runtime_world::facade::WorldProgramActivationHistory;
pub mod authored_key;
pub mod capture;
pub mod durable_witness;
pub mod expiry;
pub mod native_source;
pub mod original_join;
pub mod original_observation;
pub mod pin_join;
pub mod prepared;
pub mod ready;
pub mod recovery;

impl WorthQueryPrimaryGraphProvider {
    /// A correction commit's material; a correction commits as its group's primary writer.
    pub fn admit_correction_lineage_material(
        &self,
        commit: &worth_relational::facade::history::RelationalCommitReceipt,
    ) -> Result<(), Denial> {
        self.correction_material
            .get()
            .ok_or(Denial::AdmissionRequired)?
            .admit_original_metadata(commit, crate::domain_computation::application_aftermath::WorthQueryCorrectionSourceMember::Primary)
    }
    pub fn has_captured_correction_material(
        &self,
        original: worth_foundational::facade::DurableCompositePublicationAttemptIdentity,
    ) -> bool {
        self.correction_material
            .get()
            .is_some_and(|owner| owner.has_captured_original(original))
    }
    pub fn settle_correction_pins_for_close(
        &self,
        request: &ExecutionRequest<'_, '_>,
        inspection: &worth_runtime_world::facade::RuntimeWorldInspectionPort,
    ) -> Result<(), Denial> {
        let Some(owner) = self.correction_material.get() else {
            return Ok(());
        };
        let originals = inspection
            .observe_program_activation_history(request)
            .map_err(|_| Denial::Unavailable)?;
        owner.abandon_released(&originals, request)
    }
    pub fn observe_correction_material(
        &self,
        commit: &worth_relational::facade::history::RelationalCommitReceipt,
        member: crate::domain_computation::application_aftermath::WorthQueryCorrectionSourceMember,
    ) -> Result<
        crate::domain_computation::primary_graph::retained_correction::RetainedCorrectionMaterial,
        Denial,
    > {
        self.correction_material
            .get()
            .ok_or(Denial::AdmissionRequired)?
            .observe_original(commit, member)
    }
    /// Ready retained writers for a bounded correction history listing.
    pub fn retained_correction_history_candidates(
        &self,
    ) -> Result<
        Vec<crate::domain_computation::primary_graph::retained_correction::history_candidates::RetainedCorrectionHistoryCandidate>,
        Denial,
    >{
        self.correction_material
            .get()
            .ok_or(Denial::AdmissionRequired)?
            .ready_history_candidates()
    }
    /// A recorded correction's capture; a correction commits as its group's primary writer.
    pub fn observe_correction_capture_for_replay(
        &self,
        request: &ExecutionRequest<'_, '_>,
        commit: &worth_relational::facade::history::RelationalCommitReceipt,
        inspection: &worth_runtime_world::facade::RuntimeWorldInspectionPort,
    ) -> Result<
        Arc<crate::domain_computation::primary_graph::retained_correction::record::MaterialCapture>,
        Denial,
    > {
        let owner = self
            .correction_material
            .get()
            .ok_or(Denial::AdmissionRequired)?;
        owner.observe_capture_for_replay(commit, crate::domain_computation::application_aftermath::WorthQueryCorrectionSourceMember::Primary, || {
            inspection
                .observe_program_activation_history(request)
                .map_err(|_| Denial::Unavailable)
        })
    }
    pub fn readmit_correction_material(
        &self,
        medium: Arc<dyn QueryCorrectionsMedium>,
        originals: &WorldProgramActivationHistory,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<worth_proof::AdmittedHomeQueryCorrections, Denial> {
        if let Some(owner) = self.correction_material.get() {
            owner.admit_medium(medium.as_ref())?;
            owner.abandon_released(originals, request)?;
            owner.discharge_unperformed(Some(originals), request)?;
            return owner.admission();
        }
        let owner = self.graph.with_runtime(request, |native| {
            CorrectionMaterialOwner::readmit(medium, request, |catalog| {
                original_join::admit(request, native, self, originals, catalog)
            })
        })?;
        self.correction_material
            .set(owner)
            .map_err(|_| Denial::AdmissionAlreadyRecorded)?;
        let owner = self
            .correction_material
            .get()
            .expect("same installed material receiver");
        owner.abandon_released(originals, request)?;
        owner.discharge_unperformed(Some(originals), request)?;
        owner.admission()
    }
}
