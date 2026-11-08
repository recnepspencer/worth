//! Query-defined opaque transport and actual publication observations.
use super::*;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::HomeIncarnation;

#[derive(Debug)]
pub enum QueryObligationPublicationOutcome {
    Pending,
    Committed(worth_proof::DurablyCommitted<worth_proof::CommittedHomeQueryObligation>),
    ProvenNoEffect(worth_proof::NoEffectHomeQueryObligation),
    Refused(QueryProgramSupportDenial),
    Unresolved,
}

/// Trusted Home adapter. Descriptive identity or an arbitrary implementation
/// does not authenticate physical Home custody or construct application service.
pub trait QueryObligationsMedium: std::fmt::Debug + Send + Sync {
    fn authority(&self) -> &worth_proof::HomeQueryObligationsAuthority;
    fn home(&self) -> HomeIdentity;
    fn incarnation(&self) -> HomeIncarnation;
    fn read_frame(
        &self,
        request: &ExecutionRequest<'_, '_>,
        position: u64,
    ) -> Result<Option<Vec<u8>>, QueryProgramSupportDenial>;
    fn advance_publication(
        &self,
        request: &ExecutionRequest<'_, '_>,
        frame: &QueryProgramSupportFrame,
    ) -> QueryObligationPublicationOutcome;
    fn acknowledge_no_effect(
        &self,
        proof: worth_proof::NoEffectHomeQueryObligation,
    ) -> Result<(), QueryProgramSupportDenial>;
}
