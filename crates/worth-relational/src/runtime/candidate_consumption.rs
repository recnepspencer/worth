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
        crate::authority::commit::pipeline::publish_prepared_authoritative_commit(
            execution, self, candidate,
        )
    }
}
