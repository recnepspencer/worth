use crate::runtime::{RelationalPreparationOwnerBinding, RelationalPreparationRuntime};
use crate::transactions::data::TransactionCommitError;

/// Cloneable transaction-preparation service bound to one live relational
/// runtime owner.
///
/// Clones share the runtime's narrow preparation authorities. Candidate
/// settlement remains exclusively owned by [`crate::mvcc::RelationalPublicationPort`].
#[derive(Debug, Clone)]
pub struct RelationalPreparationPort {
    binding: RelationalPreparationOwnerBinding,
}

impl RelationalPreparationPort {
    pub(crate) fn new(binding: RelationalPreparationOwnerBinding) -> Self {
        Self { binding }
    }

    /// Validate a branch-bound transaction and register its immutable
    /// publication candidate without requiring exclusive runtime access.
    pub fn prepare_branch_transaction(
        &self,
        transaction: crate::mvcc::BranchBoundRelationalTransaction,
    ) -> Result<crate::mvcc::PreparedRelationalCommitCandidate, TransactionCommitError> {
        let _operation = self.admit_operation()?;
        let configuration = self.binding.configuration_binding();
        let epoch = configuration.operation();
        let runtime = self.binding.runtime_snapshot_from(&epoch);
        let proposal = runtime
            .validate_branch_transaction(transaction)
            .map_err(attach_validation_rejection)?;
        self.prepare_validated_proposal_inner(&runtime, proposal)
    }

    pub(crate) fn prepare_validated_proposal(
        &self,
        proposal: crate::mvcc::ValidatedRelationalProposal,
    ) -> Result<crate::mvcc::PreparedRelationalCommitCandidate, TransactionCommitError> {
        let _operation = self.admit_operation()?;
        let configuration = self.binding.configuration_binding();
        let epoch = configuration.operation();
        let runtime = self.binding.runtime_snapshot_from(&epoch);
        self.prepare_validated_proposal_inner(&runtime, proposal)
    }

    /// Consume a prepared candidate without publishing it and release its
    /// owner-governed registration.
    pub fn discard_prepared_candidate(
        &self,
        candidate: crate::mvcc::PreparedRelationalCommitCandidate,
    ) -> Result<crate::mvcc::DiscardedRelationalCommitCandidate, TransactionCommitError> {
        let _operation = self.admit_operation()?;
        let runtime = self.binding.runtime_snapshot();
        self.ensure_candidate_owner(&runtime, &candidate)?;
        let discarded = crate::mvcc::DiscardedRelationalCommitCandidate::from_candidate(&candidate);
        runtime.history.record_candidate_discard(candidate.branch());
        drop(candidate);
        Ok(discarded)
    }

    fn prepare_validated_proposal_inner(
        &self,
        runtime: &RelationalPreparationRuntime,
        proposal: crate::mvcc::ValidatedRelationalProposal,
    ) -> Result<crate::mvcc::PreparedRelationalCommitCandidate, TransactionCommitError> {
        let proposal = runtime.revalidate_proposal_for_publication(proposal)?;
        crate::authority::commit::pipeline::prepare_authoritative_commit(
            runtime,
            crate::authority::commit::pipeline::AuthoritativeCommitContext::from_validated_proposal(
                proposal,
            ),
        )
    }

    fn admit_operation(
        &self,
    ) -> Result<crate::runtime::AdmittedRelationalRuntimeOperation, TransactionCommitError> {
        let runtime = self.binding.runtime_snapshot();
        runtime.admit_operation().ok_or_else(|| {
            TransactionCommitError::publication_denied(
                crate::mvcc::RelationalPublicationDenial::OwnerUnavailable {
                    runtime_instance_id: runtime.runtime_instance_id(),
                },
            )
        })
    }

    fn ensure_candidate_owner(
        &self,
        runtime: &RelationalPreparationRuntime,
        candidate: &crate::mvcc::PreparedRelationalCommitCandidate,
    ) -> Result<(), TransactionCommitError> {
        if candidate.runtime_instance_id() == runtime.runtime_instance_id() {
            return Ok(());
        }
        Err(TransactionCommitError::conflict(
            crate::transactions::data::CommitConflict::new(
                crate::transactions::data::ConflictClass::ForeignRuntime {
                    expected_runtime_instance_id: runtime.runtime_instance_id(),
                    actual_runtime_instance_id: candidate.runtime_instance_id(),
                },
            ),
        ))
    }
}

fn attach_validation_rejection(error: TransactionCommitError) -> TransactionCommitError {
    let mut commit_log = crate::transactions::data::CommitLog::new();
    let phase = crate::transactions::data::CommitPhase::DraftPreparation;
    commit_log.begin_phase(phase);
    match &error {
        TransactionCommitError::Conflict { error, .. } => {
            commit_log.record_rejection(phase, Some(error.code()), None, error.detail());
        }
        TransactionCommitError::Publication { error, .. } => {
            commit_log.record_rejection(phase, None, Some(error.stage), error.detail.clone());
        }
        TransactionCommitError::Preparation { error, .. } => {
            commit_log.record_rejection(phase, Some(error.code()), None, error.detail());
        }
        TransactionCommitError::Interrupted { .. }
        | TransactionCommitError::PublicationDenied { .. }
        | TransactionCommitError::PublicationDeferred { .. }
        | TransactionCommitError::PublicationFailed { .. } => {
            commit_log.record_rejection(phase, None, None, error.detail());
        }
        TransactionCommitError::PerformedButDurabilityDeferred { .. } => {}
    }
    error.with_commit_log(commit_log)
}
