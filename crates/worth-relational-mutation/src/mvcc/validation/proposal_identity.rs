use crate::contracts::branch::reference::observation::RelationalBranchReferenceObservation;
use crate::contracts::branch::version::RelationalBranchVersion;
use crate::contracts::identity::data::primitives::VersionId;
use crate::contracts::transactions::data::primitives::TransactionId;

/// Runtime-local identity for one owner-issued mutation proposal.
///
/// The fields are deliberately private and there is no public constructor,
/// deserialization, or authority implementation. Only the runtime sequence
/// owner can issue this identity, and the commit pipeline may move it as
/// evidence for the exact proposal it validated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationalMutationProposalIdentity {
    omitted_publication_appendix: bool,
    runtime_instance_id: u64,
    ordinal: u64,
    transaction_id: TransactionId,
    branch_observation: RelationalBranchReferenceObservation,
    branch_version: RelationalBranchVersion,
    proposed_version_id: VersionId,
    commit_id: crate::contracts::history::data::coordinates::CommitId,
    source_interpretation: crate::validation::candidate_source::NativeSourceInterpretationIdentity,
}

impl RelationalMutationProposalIdentity {
    pub(crate) fn omitted_publication_appendix(&self) -> bool {
        self.omitted_publication_appendix
    }

    pub fn omits_publication_appendix(&self) -> bool {
        self.omitted_publication_appendix
    }

    pub(crate) fn same_proposal_sequence(&self, other: &Self) -> bool {
        self.omitted_publication_appendix == other.omitted_publication_appendix
            && self.runtime_instance_id == other.runtime_instance_id
            && self.ordinal == other.ordinal
            && self.transaction_id == other.transaction_id
            && self.branch_observation == other.branch_observation
            && self.branch_version == other.branch_version
            && self.proposed_version_id == other.proposed_version_id
            && self.commit_id == other.commit_id
    }

    pub(crate) const fn commit_id(&self) -> crate::contracts::history::data::coordinates::CommitId {
        self.commit_id
    }

    /// Only owner-admitted suspension leaves logical truth unchanged while
    /// removing physical payloads. Ordinary and reconstruction proposals have
    /// no source-view permission.
    pub(crate) fn suspension_source_version(&self) -> Option<VersionId> {
        self.source_interpretation
            .owner()?
            .suspension_source_version()
    }

    pub(crate) fn source_interpretation(
        &self,
    ) -> Option<
        std::sync::Arc<crate::validation::candidate_source::NativeManagedSourceInterpretation>,
    > {
        self.source_interpretation.owner()
    }

    pub const fn runtime_instance_id(&self) -> u64 {
        self.runtime_instance_id
    }

    pub const fn ordinal(&self) -> u64 {
        self.ordinal
    }

    pub const fn transaction_id(&self) -> TransactionId {
        self.transaction_id
    }

    pub fn branch_observation(&self) -> &RelationalBranchReferenceObservation {
        &self.branch_observation
    }

    pub const fn branch_version(&self) -> RelationalBranchVersion {
        self.branch_version
    }

    pub const fn proposed_version_id(&self) -> VersionId {
        self.proposed_version_id
    }
}

/// The reserved runtime proposal sequence and its exact source binding.
pub struct MutationProposalIdentityParts {
    pub omitted_publication_appendix: bool,
    pub runtime_instance_id: u64,
    pub ordinal: u64,
    pub transaction_id: TransactionId,
    pub branch_observation: RelationalBranchReferenceObservation,
    pub branch_version: RelationalBranchVersion,
    pub proposed_version_id: VersionId,
    pub commit_id: crate::contracts::history::data::coordinates::CommitId,
    pub source_interpretation:
        crate::validation::candidate_source::NativeSourceInterpretationIdentity,
}

pub fn bind_mutation_proposal_identity(
    parts: MutationProposalIdentityParts,
) -> RelationalMutationProposalIdentity {
    RelationalMutationProposalIdentity {
        omitted_publication_appendix: parts.omitted_publication_appendix,
        runtime_instance_id: parts.runtime_instance_id,
        ordinal: parts.ordinal,
        transaction_id: parts.transaction_id,
        branch_observation: parts.branch_observation,
        branch_version: parts.branch_version,
        proposed_version_id: parts.proposed_version_id,
        commit_id: parts.commit_id,
        source_interpretation: parts.source_interpretation,
    }
}

#[doc(hidden)]
pub trait RelationalMutationProposalIdentityOwnerPort: Sized {
    fn same_proposal_sequence(&self, other: &Self) -> bool;

    fn commit_id(&self) -> crate::contracts::history::data::coordinates::CommitId;

    fn source_interpretation(
        &self,
    ) -> Option<
        std::sync::Arc<crate::validation::candidate_source::NativeManagedSourceInterpretation>,
    >;
}

impl RelationalMutationProposalIdentityOwnerPort for RelationalMutationProposalIdentity {
    fn same_proposal_sequence(&self, other: &Self) -> bool {
        self.same_proposal_sequence(other)
    }

    fn commit_id(&self) -> crate::contracts::history::data::coordinates::CommitId {
        self.commit_id()
    }

    fn source_interpretation(
        &self,
    ) -> Option<
        std::sync::Arc<crate::validation::candidate_source::NativeManagedSourceInterpretation>,
    > {
        self.source_interpretation()
    }
}
