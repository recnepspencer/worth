//! Preserve storage refusal as a pre-effect preparation outcome.
use crate::branch::AdmittedRelationalBranchBasis;
use crate::storage::residency::RelationalResidencyDemandFailure;
use crate::transactions::data::{CommitPreparationError, TransactionCommitError};

pub(crate) fn storage_read_denial(
    basis: &AdmittedRelationalBranchBasis,
    failure: RelationalResidencyDemandFailure,
) -> TransactionCommitError {
    match failure {
        RelationalResidencyDemandFailure::Content(cause) => {
            TransactionCommitError::execution(crate::transactions::data::CommitExecutionDenial {
                kind: crate::transactions::data::CommitExecutionDenialKind::Cause(
                    cause.into_execution_cause(),
                ),
                partition_identity: None,
            })
        }
        RelationalResidencyDemandFailure::StorageUnavailable(failure) => {
            if let crate::storage::data::RelationalStorageUnavailableCause::CorruptCurrentOwnership{publication,source,record}=failure.cause {
                let mut error=crate::publication::data::PublicationError::authored_retirement(
                    crate::publication::data::AuthoredRetirementRefusal::CorruptCurrentOwnership{publication},crate::transactions::data::RecordRef::Entity(source));
                if source!=record {error.context.affected_records.push(crate::transactions::data::RecordRef::Entity(record));}
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
            TransactionCommitError::execution(crate::transactions::data::CommitExecutionDenial {
                kind: crate::transactions::data::CommitExecutionDenialKind::Cause(cause),
                partition_identity: None,
            })
        }
    }
}
