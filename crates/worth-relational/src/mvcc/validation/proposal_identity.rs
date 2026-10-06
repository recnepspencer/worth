use crate::branch::{RelationalBranchReferenceObservation, RelationalBranchVersion};
use crate::identity::data::VersionId;
use crate::transactions::data::TransactionId;

/// Runtime-local identity for one owner-issued mutation proposal.
///
/// The fields are deliberately private and there is no public constructor,
/// deserialization, or authority implementation. Only the runtime sequence
/// owner can issue this identity, and the commit pipeline may move it as
/// evidence for the exact proposal it validated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationalMutationProposalIdentity {
    runtime_instance_id: u64,
    ordinal: u64,
    transaction_id: TransactionId,
    branch_observation: RelationalBranchReferenceObservation,
    branch_version: RelationalBranchVersion,
    proposed_version_id: VersionId,
    suspension_source_version: Option<VersionId>,
}

impl RelationalMutationProposalIdentity {
    /// Only owner-admitted suspension leaves logical truth unchanged while
    /// removing physical payloads. Ordinary and reconstruction proposals have
    /// no source-view permission.
    pub(crate) const fn suspension_source_version(&self) -> Option<VersionId> {
        self.suspension_source_version
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

impl crate::runtime::RelationalPreparationRuntime {
    pub(crate) fn issue_mutation_proposal_identity(
        &self,
        transaction_id: TransactionId,
        validation_input: &crate::mvcc::RelationalTransactionValidationInput,
    ) -> Result<RelationalMutationProposalIdentity, crate::transactions::data::TransactionCommitError>
    {
        let proposed_version_id = self.history.reserve_version_id().ok_or_else(|| {
            crate::transactions::data::TransactionCommitError::publication(
                crate::publication::data::PublicationError::new(
                    crate::publication::bundle::PublicationStage::BundleAssembly,
                    "canonical version sequence overflow",
                ),
            )
        })?;
        let ordinal = self.services.next_proposal_ordinal().ok_or_else(|| {
            crate::transactions::data::TransactionCommitError::preparation(
                crate::transactions::data::CommitPreparationError::proposal_identity_exhausted(
                    validation_input.target_branch().clone(),
                    proposed_version_id,
                ),
            )
        })?;
        let suspension_source_version = matches!(
            validation_input.intent().materialization_mode(),
            Some(crate::mvcc::RelationalMaterializationTransactionMode::Suspend)
        )
        .then(|| validation_input.basis().observation().version_id());
        Ok(RelationalMutationProposalIdentity {
            runtime_instance_id: self.services.runtime_instance_id(),
            ordinal,
            transaction_id,
            branch_observation: validation_input.basis().reference().clone(),
            branch_version: validation_input.basis().truth_version(),
            proposed_version_id,
            suspension_source_version,
        })
    }
}
