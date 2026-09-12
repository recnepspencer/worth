//! Validated mutation owner and invariant evidence.

use crate::authority::commit::phases::prepare::PreparedWorkingStateScope;
use crate::branch::RelationalBranchVersion;
use crate::history::data::{BranchId, CommitId};
use crate::identity::data::VersionId;
use crate::transactions::data::CommitValidationSummary;
use crate::validation::engine::InvariantExecutionResult;

/// Owner-minted evidence that Relational evaluated the exact proposed mutation
/// through its installed commit-boundary, mutation-sensitive, and publication
/// invariant families.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationalMutationInvariantEvidence {
    pub(crate) branch: BranchId,
    pub(crate) proposed_version: VersionId,
    pub(crate) proposal_identity: super::proposal_identity::RelationalMutationProposalIdentity,
    pub(crate) summary: CommitValidationSummary,
    pub(crate) custom_invariant_execution_receipts:
        std::sync::Arc<[super::CustomInvariantExecutionReceipt]>,
}

impl RelationalMutationInvariantEvidence {
    pub const fn branch(&self) -> &BranchId {
        &self.branch
    }

    pub const fn proposed_version(&self) -> VersionId {
        self.proposed_version
    }

    pub fn proposal_identity(
        &self,
    ) -> &super::proposal_identity::RelationalMutationProposalIdentity {
        &self.proposal_identity
    }

    pub const fn summary(&self) -> CommitValidationSummary {
        self.summary
    }

    pub fn custom_invariant_execution_receipts(&self) -> &[super::CustomInvariantExecutionReceipt] {
        &self.custom_invariant_execution_receipts
    }
}

/// Move-only Relational authority for one invariant-validated proposed state.
///
/// Construction is private to `RelationalTransaction::validate`. Committing it
/// rechecks the exact branch-qualified validation basis before publication.
pub struct ValidatedRelationalProposal {
    pub(crate) mutation_authority: crate::branch::RelationalBranchMutationAuthority,
    pub(crate) transaction_id: crate::transactions::data::TransactionId,
    pub(crate) validation_input: crate::mvcc::RelationalTransactionValidationInput,
    pub(crate) prepared: PreparedWorkingStateScope,
    pub(crate) proposed_working_state: crate::runtime::WorkingState,
    pub(crate) commit_boundary: InvariantExecutionResult,
    pub(crate) mutation_sensitive: InvariantExecutionResult,
    pub(crate) snapshot_publication: InvariantExecutionResult,
    pub(crate) evidence: RelationalMutationInvariantEvidence,
    pub(crate) proposal_identity: super::proposal_identity::RelationalMutationProposalIdentity,
    pub(crate) validated_against_commit: Option<CommitId>,
    pub(crate) validated_against_version: VersionId,
    pub(crate) validated_against_branch_version: RelationalBranchVersion,
    pub(crate) custom_invariant_generation: u64,
    pub(crate) batch_count: usize,
    pub(crate) strategy_commit_artifacts:
        Option<crate::commit_strategies::data::StrategyCommitArtifactBundle>,
    pub(crate) strategy_bulk_mutation_batch:
        Option<crate::transactions::data::ProvenanceCompleteBulkMutationBatch>,
    pub(crate) validation_complexity_delta: crate::performance::data::RuntimeComplexityCounters,
}

impl std::fmt::Debug for ValidatedRelationalProposal {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ValidatedRelationalProposal")
            .field("transaction_id", &self.transaction_id)
            .field("proposal_identity", &self.proposal_identity)
            .field("validated_against_commit", &self.validated_against_commit)
            .field("validated_against_version", &self.validated_against_version)
            .field(
                "has_strategy_decoration",
                &self.strategy_commit_artifacts.is_some(),
            )
            .finish_non_exhaustive()
    }
}

impl ValidatedRelationalProposal {
    pub const fn invariant_evidence(&self) -> &RelationalMutationInvariantEvidence {
        &self.evidence
    }

    pub fn footprint(&self) -> &crate::mvcc::RelationalTransactionFootprint {
        &self.prepared.footprint
    }

    pub fn strategy_commit_artifacts(
        &self,
    ) -> Option<&crate::commit_strategies::data::StrategyCommitArtifactBundle> {
        self.strategy_commit_artifacts.as_ref()
    }

    pub fn proposal_identity(
        &self,
    ) -> &super::proposal_identity::RelationalMutationProposalIdentity {
        &self.proposal_identity
    }

    pub fn validated_against_commit_id(&self) -> Option<CommitId> {
        self.validated_against_commit
    }

    pub fn validated_against_version_id(&self) -> VersionId {
        self.validated_against_version
    }

    pub fn validation_summary(&self) -> CommitValidationSummary {
        self.evidence.summary()
    }

    pub fn custom_invariant_execution_receipts(&self) -> &[super::CustomInvariantExecutionReceipt] {
        self.evidence.custom_invariant_execution_receipts()
    }
}

pub(crate) struct StrategyProposalDecoration {
    pub(crate) artifacts: crate::commit_strategies::data::StrategyCommitArtifactBundle,
    pub(crate) bulk_mutation_batch:
        Option<crate::transactions::data::ProvenanceCompleteBulkMutationBatch>,
}

#[cfg(test)]
mod tests {
    use crate::mvcc::RelationalBranchTransactionAdmissionDenial;
    use crate::tests::support::{create_entity, runtime_with_test_schema};
    use crate::transactions::data::{CommitConflict, ConflictClass, TransactionCommitError};

    #[test]
    fn stale_owner_binding_is_rejected_during_transaction_admission() {
        let runtime = runtime_with_test_schema();
        let identity = runtime.main_branch_identity();
        let stale_options = runtime
            .transaction_validation_input_for(&identity)
            .expect("main branch owner binding");
        let _ = create_entity(&runtime, "head-advance");

        assert_eq!(
            runtime
                .begin_branch_transaction(stale_options.basis(), stale_options.intent().clone())
                .unwrap_err(),
            RelationalBranchTransactionAdmissionDenial::StaleBasis
        );
    }

    #[test]
    fn owner_rechecks_head_after_validation_before_commit() {
        let runtime = runtime_with_test_schema();
        let identity = runtime.main_branch_identity();
        let validation_input = runtime
            .transaction_validation_input_for(&identity)
            .expect("main branch owner binding");
        let candidate = runtime
            .begin_branch_transaction(validation_input.basis(), validation_input.intent().clone())
            .expect("current basis belongs to the same runtime")
            .validate(&runtime)
            .expect("head is current at validation");
        let candidate_ordinal = candidate.proposal_identity.ordinal();

        let _ = create_entity(&runtime, "post-validation-advance");
        let denied = runtime
            .prepare_validated_proposal(candidate)
            .expect_err("Relational must close the validate/commit race");

        assert!(matches!(
            denied,
            TransactionCommitError::Conflict {
                error: CommitConflict {
                    class: ConflictClass::StaleValidationBasis { .. },
                    ..
                },
                ..
            }
        ));

        let fresh_options = runtime
            .transaction_validation_input_for(&runtime.main_branch_identity())
            .expect("fresh main binding");
        let fresh = runtime
            .begin_branch_transaction(fresh_options.basis(), fresh_options.intent().clone())
            .expect("fresh basis belongs to the same runtime")
            .validate(&runtime)
            .expect("fresh validation remains admissible");
        assert_eq!(
            fresh.proposal_identity.ordinal(),
            candidate_ordinal + 2,
            "stale same-branch denial must not consume a proposal ordinal"
        );
    }
}
