use crate::runtime::RelationalRuntime;
use crate::transactions::data::{CommitResult, TransactionCommitError};

impl RelationalRuntime {
    /// Requestless cleanup: expired retained resources must be released even after cancellation.
    pub fn reap_expired_prepared_candidates(&self) -> usize {
        self.publication_binding().reap_expired_candidates()
    }

    pub fn discard_prepared_candidate(
        &self,
        candidate: crate::mvcc::PreparedRelationalCommitCandidate,
    ) -> Result<crate::mvcc::DiscardedRelationalCommitCandidate, TransactionCommitError> {
        self.preparation_port()
            .discard_prepared_candidate(candidate)
    }

    pub(crate) fn publish_prepared_candidate(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        candidate: crate::mvcc::PreparedRelationalCommitCandidate,
    ) -> Result<CommitResult, TransactionCommitError> {
        crate::authority::commit::pipeline::publish_prepared_authoritative_commit(
            execution, self, candidate,
        )
    }
}
