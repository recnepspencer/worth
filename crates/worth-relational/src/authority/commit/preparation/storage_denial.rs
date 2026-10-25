//! Preserve storage refusal as a pre-effect preparation outcome.
use crate::branch::AdmittedRelationalBranchBasis;
use crate::storage::residency::RelationalResidencyDemandFailure;
use crate::transactions::data::{CommitPreparationError, TransactionCommitError};

pub(crate) fn storage_read_denial(
    basis: &AdmittedRelationalBranchBasis,
    failure: RelationalResidencyDemandFailure,
) -> TransactionCommitError {
    match failure {
        RelationalResidencyDemandFailure::StorageUnavailable(failure) => {
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
