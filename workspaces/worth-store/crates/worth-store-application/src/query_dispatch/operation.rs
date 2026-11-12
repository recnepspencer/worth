//! The narrow owned operation the actual Home lends Host for Query's dispatch journal.
use super::*;
use crate::query_journal::QueryJournalPublicationOutcome as Physical;

/// A Home-issued handle over exactly one journal. Every handle shares the
/// Home's one journal owner and its one original publication slot, so a second
/// handle adds no second writer. It names no other journal, exposes no
/// physical runtime, and never keeps the Home's serving alive: after the
/// issuing Home releases its serving it answers `OwnerUnavailable`.
#[derive(Clone, Debug)]
pub struct ApplicationHomeDispatchJournalOperation {
    journal: Arc<ApplicationHomeDispatchJournal>,
}
impl ApplicationHomeDispatchJournalOperation {
    /// Receiver-owned stop at a prepared original, before any physical effect.
    /// The ordinary Store endpoint still derives and issues its actual fate.
    pub fn stop_next_prepared_publication_for_test(&self) {
        self.journal
            .journal
            .stop_next_prepared_publication_for_test();
    }
    pub(crate) fn issue(journal: &Arc<ApplicationHomeDispatchJournal>) -> Self {
        Self {
            journal: Arc::clone(journal),
        }
    }
    pub fn authority(&self) -> &worth_proof::HomeQueryDispatchAuthority {
        &self.journal.authority
    }
    pub fn home(&self) -> HomeIdentity {
        self.journal.home
    }
    pub fn incarnation(&self) -> HomeIncarnation {
        self.journal.incarnation
    }
    /// `None` proves the position is the unwritten successor of the durable head.
    pub fn read_frame(
        &self,
        request: &ExecutionRequest<'_, '_>,
        position: u64,
        maximum_bytes: u64,
    ) -> Result<Option<Vec<u8>>, ApplicationHomeDispatchJournalDenial> {
        self.journal
            .journal
            .read_frame(request, position, maximum_bytes)
            .map_err(denial)
    }
    /// Drive the one original publication of `frame` at `position`. The same
    /// original task stays in this Home across refusals and invoking requests.
    pub fn advance_publication(
        &self,
        request: &ExecutionRequest<'_, '_>,
        position: u64,
        frame: &[u8],
    ) -> ApplicationHomeDispatchPublication {
        use ApplicationHomeDispatchPublication as Outcome;
        match self.journal.journal.advance(request, position, frame) {
            Physical::Pending => Outcome::Pending,
            Physical::Committed { position, digest } => Outcome::Committed(
                self.journal
                    .owner
                    .acknowledge_query_dispatch_commit(position, digest),
            ),
            Physical::ProvenNoEffect { position, digest } => Outcome::ProvenNoEffect(
                self.journal
                    .owner
                    .acknowledge_query_dispatch_no_effect(position, digest),
            ),
            Physical::Refused(cause) => Outcome::Refused(denial(cause)),
            Physical::Unresolved => Outcome::Unresolved,
        }
    }
    /// Release the original task only for its own proven no-effect witness.
    pub fn acknowledge_no_effect(
        &self,
        proof: worth_proof::NoEffectHomeQueryDispatch,
    ) -> Result<(), ApplicationHomeDispatchJournalDenial> {
        proof
            .subject()
            .admit_to(&self.journal.authority)
            .map_err(incarnation_denial)?;
        self.journal
            .journal
            .acknowledge_no_effect(proof.subject().position(), proof.subject().payload_digest())
            .map_err(denial)
    }
}
