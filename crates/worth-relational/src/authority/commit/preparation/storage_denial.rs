//! Preserve storage refusal as a pre-effect preparation outcome.
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::storage::residency::RelationalResidencyDemandFailure;
use crate::transactions::data::outcomes::{
    commit_preparation::CommitPreparationError,
    transaction_commit_error::{
        CommitExecutionDenial, CommitExecutionDenialKind, TransactionCommitError,
    },
};

pub(crate) fn storage_read_denial(
    basis: &AdmittedRelationalBranchBasis,
    failure: RelationalResidencyDemandFailure,
) -> TransactionCommitError {
    match failure {
        RelationalResidencyDemandFailure::Content(cause) => {
            TransactionCommitError::execution(CommitExecutionDenial {
                kind: CommitExecutionDenialKind::Cause(cause.into_execution_cause()),
                partition_identity: None,
            })
        }
        RelationalResidencyDemandFailure::StorageUnavailable(failure) => {
            if let crate::storage::data::storage_unavailable::RelationalStorageUnavailableCause::CorruptCurrentOwnership{publication,source,record}=failure.cause {
                let mut error=crate::publication::data::publication_error::PublicationError::authored_retirement(
                    crate::publication::data::authored_retirement_refusal::AuthoredRetirementRefusal::CorruptCurrentOwnership{publication},crate::transactions::data::primitives::RecordRef::Entity(source));
                if source!=record {error.context.affected_records.push(crate::transactions::data::primitives::RecordRef::Entity(record));}
                return TransactionCommitError::publication(error);
            }
            TransactionCommitError::preparation(CommitPreparationError::storage_unavailable(
                basis.identity().branch_id().clone(),
                basis.observation().commit_id().map(|id| id.0),
                basis.observation().version_id(),
                failure,
            ))
        }
        RelationalResidencyDemandFailure::Request(cause) => {
            TransactionCommitError::execution(CommitExecutionDenial {
                kind: CommitExecutionDenialKind::Cause(cause),
                partition_identity: None,
            })
        }
    }
}
