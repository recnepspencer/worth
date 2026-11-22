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
pub mod expiry;
pub mod native_source;
pub mod original_join;
pub mod original_observation;
pub mod pin_join;
pub mod prepared;
pub mod ready;
pub mod recovery;

impl WorthQueryPrimaryGraphProvider {
    pub fn admit_correction_lineage_material(
        &self,
        commit: &worth_relational::facade::history::RelationalCommitReceipt,
    ) -> Result<(), Denial> {
        self.correction_material
            .get()
            .ok_or(Denial::AdmissionRequired)?
            .admit_original_metadata(commit)
    }
    pub fn observe_correction_material(
        &self,
        commit: &worth_relational::facade::history::RelationalCommitReceipt,
    ) -> Result<
        crate::domain_computation::primary_graph::retained_correction::RetainedCorrectionMaterial,
        Denial,
    > {
        self.correction_material
            .get()
            .ok_or(Denial::AdmissionRequired)?
            .observe_original(commit)
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
