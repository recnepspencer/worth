//! Duplicate only a real issued phase at the closed corruption receiver.
use worth_relational_publication::internal_port::mvcc::publication::application_publication::coupled_assembly::continuation::custody::RelationalCoupledProducerContinuationOwnerPort as _;
use super::{ExecutionRequest, RelationalAuthoredCoupledCandidate, TransactionCommitError};
impl RelationalAuthoredCoupledCandidate<'_> {
    pub(in crate::mvcc::publication::application_publication::coupled_assembly) fn replay_continuation_probe(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<(), TransactionCommitError> {
        let token = self.next_producer_round(execution, self.parts.len() - 1)?;
        // Closed receiver corruption: duplicate only the real issued token.
        // No constructor or clone for this lifecycle phase reaches Query.
        let replay = token.duplicate_for_replay_probe();
        self.continuation
            .consume(token, self.identity, self.applied_parts())?;
        self.continuation
            .consume(replay, self.identity, self.applied_parts())?;
        Ok(())
    }

    /// Re-present the real writer's own commands as a continued round of the
    /// route Native just applied. Probe 7 returns every command; 8 only the
    /// commands that patch an existing record, whose values D already holds;
    /// 9 those same patches with each unsigned value moved by one.
    pub(in crate::mvcc::publication::application_publication::coupled_assembly) fn continued_round_probe(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        posture: super::super::RelationalCoupledProducerPosture,
        content: &crate::content::RelationalContentAttachments,
        batches: Vec<crate::transactions::data::primitives::WorkerIntentBatch>,
        probe: u8,
    ) -> Result<(), super::super::RelationalCoupledAssemblyFailure> {
        let round = super::application::probe_commands(&batches, probe, self.authored_source())?;
        let unchanged_content = crate::content::RelationalContentAttachments::default();
        self.append_publication(execution, posture, content, batches)?;
        let token = self.next_producer_round(execution, self.parts.len() - 1)?;
        self.verify_producer_round(
            execution,
            token,
            if probe == 7 {
                content
            } else {
                &unchanged_content
            },
            round,
        )
    }
}
