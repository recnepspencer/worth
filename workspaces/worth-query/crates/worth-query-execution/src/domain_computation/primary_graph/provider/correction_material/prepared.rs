//! Current receiver resources service port-free retained command grants.
use super::*;
use crate::domain_computation::primary_graph::{
    retained_correction::record::PreparedCorrectionSnapshot,
    WorthQueryApplicationCommitAuthorityBinding, WorthQueryPreparedCorrectionSource,
};
use worth_foundational::facade::{
    CorrectionMaterialPinIdentity, DurableCompositePublicationAttemptIdentity,
};
impl WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph) fn prepare_correction_command(
        &self,
        original: DurableCompositePublicationAttemptIdentity,
        snapshot: PreparedCorrectionSnapshot,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<CorrectionMaterialPinIdentity, Denial> {
        self.correction_material
            .get()
            .ok_or(Denial::AdmissionRequired)?
            .acquire_prepared(original, snapshot, request)
    }
    pub(in crate::domain_computation::primary_graph) fn recover_correction_command(
        &self,
        original: DurableCompositePublicationAttemptIdentity,
        key: [u8; 32],
    ) -> Result<
        Option<(
            CorrectionMaterialPinIdentity,
            Arc<PreparedCorrectionSnapshot>,
        )>,
        Denial,
    > {
        self.correction_material
            .get()
            .ok_or(Denial::AdmissionRequired)?
            .prepared_command(original, key)
    }
    pub(in crate::domain_computation::primary_graph::provider) fn transfer_correction_preparation(
        &self,
        source: &WorthQueryPreparedCorrectionSource,
        authority: &WorthQueryApplicationCommitAuthorityBinding,
        opened: &worth_runtime_world::facade::WorldOpenedPublicationAttempt,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        source
            .admit_home_receiver(opened.attempt_identity().owner_identity())
            .map_err(incarnation)?;
        let Some(identity) = source.material_pin() else {
            return Ok(());
        };
        self.correction_material
            .get()
            .ok_or(Denial::AdmissionRequired)?
            .transfer_prepared(identity, opened, authority, request)
    }
    pub(in crate::domain_computation::primary_graph::provider) fn release_correction_preparation(
        &self,
        source: &WorthQueryPreparedCorrectionSource,
        application: &super::super::WorthQueryPrimaryGraphCommittedApplication,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        let attempt = application
            .committed_product_publication()
            .publication_attempt();
        source
            .admit_home_receiver(attempt.owner_identity())
            .map_err(incarnation)?;
        let Some(identity) = source.material_pin() else {
            return Ok(());
        };
        let owner = attempt
            .owner_identity()
            .durable()
            .ok_or(Denial::ForeignHome)?;
        self.correction_material
            .get()
            .ok_or(Denial::AdmissionRequired)?
            .release_publication(
                identity,
                DurableCompositePublicationAttemptIdentity::from_recorded(owner, attempt.ordinal()),
                request,
            )
    }
}
fn incarnation(cause: worth_proof::HomeIncarnationDenial) -> Denial {
    match cause {
        worth_proof::HomeIncarnationDenial::ForeignHome => Denial::ForeignHome,
        worth_proof::HomeIncarnationDenial::StaleIncarnation => Denial::StaleIncarnation,
    }
}
