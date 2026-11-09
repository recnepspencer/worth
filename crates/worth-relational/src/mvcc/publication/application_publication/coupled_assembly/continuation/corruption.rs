//! Duplicate only a real issued phase at the closed corruption receiver.
use super::{
    ExecutionRequest, RelationalAuthoredCoupledCandidate, RelationalCoupledProducerContinuation,
    TransactionCommitError,
};
impl RelationalAuthoredCoupledCandidate<'_> {
    pub(in crate::mvcc::publication::application_publication::coupled_assembly) fn replay_continuation_probe(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<(), TransactionCommitError> {
        let token = self.next_producer_round(execution, self.parts.len() - 1)?;
        // Closed receiver corruption: duplicate only the real issued token.
        // No constructor or clone for this lifecycle phase reaches Query.
        let replay = RelationalCoupledProducerContinuation {
            identity: token.identity.clone(),
            sequence: token.sequence,
            part: token.part,
            phase: token.phase,
            source: token.source,
        };
        self.continuation
            .consume(token, self.identity, self.parts.len())?;
        self.continuation
            .consume(replay, self.identity, self.parts.len())?;
        Ok(())
    }
}
