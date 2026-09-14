use crate::history::data::CommitId;
use crate::identity::data::{EntityId, KindId, RelationId};
use crate::transactions::data::{AspectFieldPatch, RecordRef};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum RelationalMaterializationRecord {
    Entity {
        entity_id: EntityId,
        kind_id: KindId,
    },
    Relation {
        relation_id: RelationId,
        kind_id: KindId,
        source: EntityId,
        target: EntityId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationalEntityMaterialization {
    pub entity_id: EntityId,
    pub kind_id: KindId,
    pub fields: AspectFieldPatch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationalRelationMaterialization {
    pub relation_id: RelationId,
    pub kind_id: KindId,
    pub source: EntityId,
    pub target: EntityId,
    pub fields: AspectFieldPatch,
}

pub struct SuspendRelationalMaterialization;
impl worth_proof::ActionMarker for SuspendRelationalMaterialization {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RelationalMaterializationCustodyData {
    pub(crate) branch_identity: super::RelationalBranchIdentity,
    pub(crate) source_commit: CommitId,
    pub(crate) suspension_commit: CommitId,
    pub(crate) records: Vec<RelationalMaterializationRecord>,
}

#[derive(Debug)]
#[must_use = "materialization custody must be retained until reconstruction completes"]
pub struct RelationalMaterializationCustody {
    performed: worth_proof::Performed<
        SuspendRelationalMaterialization,
        super::RelationalBranchMutationAuthorityMarker,
        RelationalMaterializationCustodyData,
    >,
}

impl RelationalMaterializationCustody {
    pub(crate) fn record(data: RelationalMaterializationCustodyData) -> Self {
        Self {
            performed: worth_proof::Performed::record(
                &super::issue_relational_branch_mutation_authority(),
                data,
            ),
        }
    }

    pub fn source_commit(&self) -> CommitId {
        self.performed.outcome().source_commit
    }

    pub fn suspension_commit(&self) -> CommitId {
        self.performed.outcome().suspension_commit
    }

    pub fn records(&self) -> &[RelationalMaterializationRecord] {
        &self.performed.outcome().records
    }

    pub(crate) fn data(&self) -> &RelationalMaterializationCustodyData {
        self.performed.outcome()
    }
}

#[derive(Debug)]
pub struct RelationalMaterializationSuspension {
    pub commit: crate::transactions::data::CommitResult,
    pub custody: RelationalMaterializationCustody,
}

#[must_use = "a prepared suspension must be published or discarded"]
pub struct PreparedRelationalMaterializationSuspension {
    pub(crate) candidate: crate::mvcc::PreparedRelationalCommitCandidate,
    pub(crate) completion: RelationalMaterializationSuspensionCompletion,
}

impl PreparedRelationalMaterializationSuspension {
    pub fn into_parts(
        self,
    ) -> (
        crate::mvcc::PreparedRelationalCommitCandidate,
        RelationalMaterializationSuspensionCompletion,
    ) {
        (self.candidate, self.completion)
    }
}

pub struct RelationalMaterializationSuspensionCompletion {
    pub(crate) transaction_id: crate::transactions::data::TransactionId,
    pub(crate) branch_identity: super::RelationalBranchIdentity,
    pub(crate) source_commit: CommitId,
    pub(crate) records: Vec<RelationalMaterializationRecord>,
}

impl RelationalMaterializationSuspensionCompletion {
    pub fn complete(
        self,
        commit: crate::transactions::data::CommitResult,
    ) -> Result<RelationalMaterializationSuspension, RelationalMaterializationError> {
        if commit.transaction_id != self.transaction_id {
            return Err(RelationalMaterializationError::PublicationResultMismatch);
        }
        let custody =
            RelationalMaterializationCustody::record(RelationalMaterializationCustodyData {
                branch_identity: self.branch_identity,
                source_commit: self.source_commit,
                suspension_commit: commit.commit.commit_id,
                records: self.records,
            });
        Ok(RelationalMaterializationSuspension { commit, custody })
    }
}

#[must_use = "a prepared restoration retains materialization custody"]
pub struct PreparedRelationalRematerialization {
    pub(crate) candidate: crate::mvcc::PreparedRelationalCommitCandidate,
    pub(crate) completion: RelationalRematerializationCompletion,
    pub(crate) invariant_evidence: crate::mvcc::RelationalMutationInvariantEvidence,
}

impl PreparedRelationalRematerialization {
    pub fn into_parts(
        self,
    ) -> (
        crate::mvcc::PreparedRelationalCommitCandidate,
        RelationalRematerializationCompletion,
        crate::mvcc::RelationalMutationInvariantEvidence,
    ) {
        (self.candidate, self.completion, self.invariant_evidence)
    }
}

pub struct RelationalRematerializationCompletion {
    pub(crate) transaction_id: crate::transactions::data::TransactionId,
    pub(crate) custody: RelationalMaterializationCustody,
}

impl RelationalRematerializationCompletion {
    pub fn complete(
        self,
        commit: crate::transactions::data::CommitResult,
    ) -> Result<crate::transactions::data::CommitResult, RelationalRematerializationFailure> {
        if commit.transaction_id != self.transaction_id {
            return Err(RelationalRematerializationFailure {
                custody: self.custody,
                error: RelationalMaterializationError::PublicationResultMismatch,
            });
        }
        Ok(commit)
    }

    pub fn into_custody(self) -> RelationalMaterializationCustody {
        self.custody
    }
}

#[derive(Debug)]
pub enum RelationalMaterializationError {
    OwnerUnavailable,
    SourceCommitMismatch,
    SourceIsNotCompleteCreatePublication,
    SourceRecordUnavailable(RecordRef),
    BranchMismatch,
    SuspensionCommitMismatch,
    CandidateManifestMismatch,
    PublicationResultMismatch,
    TransactionAdmission(crate::mvcc::RelationalBranchTransactionAdmissionDenial),
    TransactionStaging(crate::mvcc::RelationalTransactionStagingDenial),
    Commit(crate::transactions::data::TransactionCommitError),
}

#[derive(Debug)]
pub struct RelationalRematerializationFailure {
    pub custody: RelationalMaterializationCustody,
    pub error: RelationalMaterializationError,
}
