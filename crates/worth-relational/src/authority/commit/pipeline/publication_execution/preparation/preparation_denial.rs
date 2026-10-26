//! Translate a pre-effect refusal using the candidate's original branch basis.
use crate::branch::AdmittedRelationalBranchBasis;
use crate::mvcc::PublicationPreparationDenial;
use crate::publication::bundle::PublicationStage;
use crate::publication::data::PublicationError;
use crate::transactions::data::{CommitLog, CommitPreparationError, TransactionCommitError};

pub(super) fn transaction_error(
    denial: PublicationPreparationDenial,
    basis: &AdmittedRelationalBranchBasis,
    commit_log: CommitLog,
) -> TransactionCommitError {
    let error = match denial {
        PublicationPreparationDenial::StorageUnavailable(failure) => {
            TransactionCommitError::preparation(CommitPreparationError::storage_unavailable(
                basis.identity().branch_id().clone(),
                basis.observation().commit_id().map(|id| id.0),
                basis.observation().version_id(),
                failure,
            ))
        }
        PublicationPreparationDenial::Invalid(detail) => TransactionCommitError::publication(
            PublicationError::new(PublicationStage::BundleAssembly, detail),
        ),
    };
    error.with_commit_log(commit_log)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::branch::RelationalBranchRootCaptureDenial;
    use crate::storage::data::{
        RelationalStorageSubject, RelationalStorageUnavailable, RelationalStorageUnavailableCause,
    };
    use crate::tests::support::{
        create_entity, runtime_with_test_schema, test_owner_begin_transaction_for_main,
    };
    use crate::transactions::data::CommitPreparationReason;

    #[test]
    fn root_storage_refusal_preserves_its_subject_cause_and_original_branch_basis() {
        let host = crate::test_execution_policy::serial_request();
        let execution = &worth_execution::ExecutionRequest::serial(&host);
        let runtime = runtime_with_test_schema();
        let entity = create_entity(execution, &runtime, "typed-root-refusal");
        let transaction = test_owner_begin_transaction_for_main(&runtime);
        let basis = &transaction.basis;
        let failure = RelationalStorageUnavailable {
            subject: RelationalStorageSubject::Region(entity.partition_id),
            cause: RelationalStorageUnavailableCause::MissingMember,
        };
        let denial = PublicationPreparationDenial::from(
            RelationalBranchRootCaptureDenial::StorageUnavailable(failure),
        );
        let error = transaction_error(denial, basis, CommitLog::default());
        let TransactionCommitError::Preparation { error, .. } = error else {
            panic!("a storage refusal must retain its preparation classification");
        };
        assert_eq!(
            error.reason(),
            CommitPreparationReason::StorageUnavailable(failure)
        );
        assert_eq!(error.branch_id(), basis.identity().branch_id());
        assert_eq!(
            error.expected_commit_id(),
            basis.observation().commit_id().map(|id| id.0)
        );
        assert_eq!(
            error.expected_version_id(),
            basis.observation().version_id()
        );
    }
}
