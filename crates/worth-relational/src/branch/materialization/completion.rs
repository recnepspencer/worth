//! Prepared and performed materialization results at the live runtime boundary.
use super::{
    source::NativeMaterializationSource, RelationalMaterializationCustody,
    RelationalMaterializationCustodyData, RelationalMaterializationRecord,
};
use crate::history::data::CommitId;
use crate::transactions::data::primitives::RecordRef;
use std::sync::Arc;

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
    pub(crate) transaction_id: crate::transactions::data::primitives::TransactionId,
    pub(crate) branch_identity: crate::branch::identity::RelationalBranchIdentity,
    pub(crate) source_commit: CommitId,
    pub(crate) records: std::sync::Arc<Vec<RelationalMaterializationRecord>>,
    pub(crate) source: Arc<NativeMaterializationSource>,
    pub(crate) producer_publications: Vec<CommitId>,
    pub(crate) producer_records:
        Vec<crate::history::data::CanonicalRequiredOutputMaterializationRecord>,
    pub(crate) producer_memory: worth_execution::ExecutionMemoryReservation,
}

impl RelationalMaterializationSuspensionCompletion {
    /// Descriptive source records captured by this original prepared suspension.
    /// Reading them grants no suspension custody or publication completion.
    pub fn source_records(&self) -> &[RelationalMaterializationRecord] {
        self.records.as_slice()
    }

    pub fn complete(
        self,
        commit: crate::transactions::data::CommitResult,
    ) -> Result<RelationalMaterializationSuspension, RelationalMaterializationError> {
        if commit.transaction_id != self.transaction_id {
            return Err(RelationalMaterializationError::PublicationResultMismatch);
        }
        let custody = RelationalMaterializationCustody::record(
            RelationalMaterializationCustodyData {
                branch_identity: self.branch_identity,
                source_commit: self.source_commit,
                suspension_commit: commit.commit.commit_id,
                records: self.records,
                source: self.source,
                producer_publications: self.producer_publications,
                producer_records: self.producer_records,
            },
            self.producer_memory,
        );
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
    pub(crate) transaction_id: crate::transactions::data::primitives::TransactionId,
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
    Read(crate::execution::denial_cause::RelationalExecutionDenialCause),
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
    Commit(crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError),
}

#[derive(Debug)]
pub struct RelationalRematerializationFailure {
    pub custody: RelationalMaterializationCustody,
    pub error: RelationalMaterializationError,
}
