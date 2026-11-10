//! Query-defined opaque dispatch-journal transport and actual publication observations.
use super::*;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::HomeIncarnation;

#[derive(Debug)]
pub enum QueryDispatchPublicationOutcome {
    Pending,
    Committed(worth_proof::DurablyCommitted<worth_proof::CommittedHomeQueryDispatch>),
    ProvenNoEffect(worth_proof::NoEffectHomeQueryDispatch),
    Refused(QueryDispatchJournalDenial),
    Unresolved,
}

/// Trusted Home adapter over the Home's own `query.dispatch` journal operation.
/// Descriptive identity or an arbitrary implementation does not authenticate
/// physical Home custody, admit dispatch, or construct application service.
pub trait QueryDispatchMedium: std::fmt::Debug + Send + Sync {
    fn authority(&self) -> &worth_proof::HomeQueryDispatchAuthority;
    fn home(&self) -> HomeIdentity;
    fn incarnation(&self) -> HomeIncarnation;
    fn read_frame(
        &self,
        request: &ExecutionRequest<'_, '_>,
        position: u64,
    ) -> Result<Option<Vec<u8>>, QueryDispatchJournalDenial>;
    fn advance_publication(
        &self,
        request: &ExecutionRequest<'_, '_>,
        frame: &QueryDispatchFrame,
    ) -> QueryDispatchPublicationOutcome;
    fn acknowledge_no_effect(
        &self,
        proof: worth_proof::NoEffectHomeQueryDispatch,
    ) -> Result<(), QueryDispatchJournalDenial>;
}
