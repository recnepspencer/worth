use crate::runtime::RelationalRuntime;
use crate::transactions::data::outcomes::commit_result::CommitResult;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;

impl RelationalRuntime {
    pub fn discard_prepared_candidate(
        &self,
        candidate: crate::mvcc::publication::candidate::PreparedRelationalCommitCandidate,
    ) -> Result<
        crate::mvcc::publication::candidate::DiscardedRelationalCommitCandidate,
        TransactionCommitError,
    > {
        self.preparation_port()
            .discard_prepared_candidate(candidate)
    }

    pub(crate) fn publish_prepared_candidate(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        candidate: crate::mvcc::publication::candidate::PreparedRelationalCommitCandidate,
    ) -> Result<CommitResult, TransactionCommitError> {
        let outcome = self
            .publication_port()
            .compare_and_publish(execution, candidate);
        let performed =
            crate::authority::commit::pipeline::performed_outcome::require_performed(outcome)?;
        self.settle_performed_publication(performed)
    }
}
