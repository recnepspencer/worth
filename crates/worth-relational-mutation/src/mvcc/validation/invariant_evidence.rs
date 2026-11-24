//! Evidence of validation for one exact proposal.
use crate::contracts::history::data::coordinates::BranchId;
use crate::contracts::identity::data::primitives::VersionId;
use crate::contracts::transactions::data::outcomes::commit_validation::CommitValidationSummary;

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

/// Results carried by the validation owner for the exact proposal it evaluated.
pub struct MutationInvariantEvidenceParts {
    pub branch: BranchId,
    pub proposed_version: VersionId,
    pub proposal_identity: super::proposal_identity::RelationalMutationProposalIdentity,
    pub summary: CommitValidationSummary,
    pub custom_invariant_execution_receipts:
        std::sync::Arc<[crate::mvcc::validation::custom_invariant_execution_receipt::CustomInvariantExecutionReceipt]>,
}

pub fn assemble_mutation_invariant_evidence(
    parts: MutationInvariantEvidenceParts,
) -> RelationalMutationInvariantEvidence {
    RelationalMutationInvariantEvidence {
        branch: parts.branch,
        proposed_version: parts.proposed_version,
        proposal_identity: parts.proposal_identity,
        summary: parts.summary,
        custom_invariant_execution_receipts: parts.custom_invariant_execution_receipts,
    }
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
