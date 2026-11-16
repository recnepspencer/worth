//! Retention joins the actual current material receiver and original support owner.
use super::*;
use crate::domain_computation::primary_graph::query_obligations::QueryObligationsMedium;
use crate::domain_computation::primary_graph::{
    query_obligations::history::QueryObligationHistory, retained_correction::MaterialPublication,
    WorthQueryCorrectionRetentionDenial as Failure,
};
impl WorthQueryPrimaryGraphProvider {
    fn material_retention(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<MaterialPublication<'_>, Denial> {
        self.correction_material
            .get()
            .ok_or(Denial::AdmissionRequired)?
            .settle_retention(request)
    }
    pub(in crate::domain_computation::primary_graph) fn readmit_retained_program_support(
        &self,
        support: &QueryObligationHistory,
        medium: Arc<dyn QueryObligationsMedium>,
        originals: &WorldProgramActivationHistory,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<worth_proof::AdmittedHomeQueryObligations, Failure> {
        let material = self.material_retention(request)?;
        support.readmit_journal(medium, originals, &material, request)
    }
    pub(in crate::domain_computation::primary_graph) fn expire_optional_correction_material(
        &self,
        originals: &WorldProgramActivationHistory,
        coordinates: &crate::domain_computation::application_aftermath::WorthQueryCorrectionSourceCoordinates,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Failure> {
        let owner = self
            .correction_material
            .get()
            .ok_or(Denial::AdmissionRequired)?;
        let material = owner.expire_at_coordinates(coordinates, originals, request)?;
        self.dispose_expired_meanings(&material, request)
    }
    pub(in crate::domain_computation::primary_graph::provider) fn settle_expired_correction_meanings(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Failure> {
        if self.correction_material.get().is_none() {
            return Ok(());
        }
        let material = self.material_retention(request)?;
        self.dispose_expired_meanings(&material, request)
    }
    fn dispose_expired_meanings(
        &self,
        material: &MaterialPublication<'_>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Failure> {
        if !material.has_expired_material() {
            return Ok(());
        }
        let support = self
            .original_meaning_support
            .get()
            .and_then(std::sync::Weak::upgrade)
            .ok_or(crate::facade::query_obligations::QueryObligationDenial::SupportUnavailable)?;
        support.dispose_expired_interpretations(material, request)
    }
}
