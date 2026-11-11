//! Only the actual Home adapter issues correction-journal publication evidence.
use super::*;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::HomeIncarnation;
#[derive(Debug)]
pub enum QueryCorrectionMaterialPublicationOutcome {
    Pending,
    Committed(worth_proof::DurablyCommitted<worth_proof::CommittedHomeQueryCorrection>),
    ProvenNoEffect(worth_proof::NoEffectHomeQueryCorrection),
    Refused(QueryCorrectionMaterialDenial),
    Unresolved,
}
/// Transport authenticates its original physical endpoint, not body meaning.
pub trait QueryCorrectionsMedium: std::fmt::Debug + Send + Sync {
    fn authority(&self) -> &worth_proof::HomeQueryCorrectionsAuthority;
    fn home(&self) -> HomeIdentity;
    fn incarnation(&self) -> HomeIncarnation;
    fn read_frame(
        &self,
        request: &ExecutionRequest<'_, '_>,
        position: u64,
        maximum_bytes: u64,
    ) -> Result<Option<Vec<u8>>, QueryCorrectionMaterialDenial>;
    fn advance_publication(
        &self,
        request: &ExecutionRequest<'_, '_>,
        frame: &QueryCorrectionMaterialFrame,
    ) -> QueryCorrectionMaterialPublicationOutcome;
    fn acknowledge_no_effect(
        &self,
        proof: worth_proof::NoEffectHomeQueryCorrection,
    ) -> Result<(), QueryCorrectionMaterialDenial>;
}
