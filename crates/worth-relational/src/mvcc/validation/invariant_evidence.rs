//! Evidence of validation for one exact proposal.
use crate::history::data::BranchId;
use crate::identity::data::VersionId;
use crate::transactions::data::outcomes::commit_validation::CommitValidationSummary;

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
        std::sync::Arc<[crate::mvcc::validation::custom_invariant_execution_receipt::CustomInvariantExecutionReceipt]>,
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

    pub fn custom_invariant_execution_receipts(&self) -> &[crate::mvcc::validation::custom_invariant_execution_receipt::CustomInvariantExecutionReceipt]{
        &self.custom_invariant_execution_receipts
    }
}
