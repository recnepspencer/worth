use crate::runtime::RelationalRuntime;
use crate::transactions::data::{CommitResult, TransactionCommitError};

impl RelationalRuntime {
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
        self.ensure_candidate_owner(&candidate)?;
        crate::authority::commit::pipeline::publish_prepared_authoritative_commit(
            execution, self, candidate,
        )
    }

    fn ensure_candidate_owner(
        &self,
        candidate: &crate::mvcc::PreparedRelationalCommitCandidate,
    ) -> Result<(), TransactionCommitError> {
        if candidate.runtime_instance_id() == self.runtime_instance_id() {
            return Ok(());
        }
        Err(TransactionCommitError::conflict(
            crate::transactions::data::CommitConflict::new(
                crate::transactions::data::ConflictClass::ForeignRuntime {
                    expected_runtime_instance_id: self.runtime_instance_id(),
                    actual_runtime_instance_id: candidate.runtime_instance_id(),
                },
            ),
        ))
    }
}
