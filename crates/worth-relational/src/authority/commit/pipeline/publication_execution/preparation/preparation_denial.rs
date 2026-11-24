//! Translate a pre-effect refusal using the candidate's original branch basis.
use worth_relational_contracts::internal_port::transactions::data::outcomes::commit_preparation::CommitPreparationErrorOwnerPort as _;
use worth_relational_contracts::internal_port::transactions::data::outcomes::transaction_commit_error::TransactionCommitErrorOwnerPort as _;
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::history::data::{BranchId, CommitId};
use crate::identity::data::VersionId;
use crate::mvcc::publication::PublicationPreparationDenial;
use crate::publication::bundle::PublicationStage;
use crate::publication::data::PublicationError;
use crate::transactions::data::commit_log::CommitLog;
use crate::transactions::data::outcomes::commit_preparation::CommitPreparationError;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;

/// Descriptive attribution for a denial on the exact selected branch.
pub(super) struct PreparationFailureBasis<'a> {
    branch_id: &'a BranchId,
    commit_id: Option<CommitId>,
    version: VersionId,
}

impl<'a> From<&'a AdmittedRelationalBranchBasis> for PreparationFailureBasis<'a> {
    fn from(basis: &'a AdmittedRelationalBranchBasis) -> Self {
        let observation = basis.observation();
        Self {
            branch_id: basis.identity().branch_id(),
            commit_id: observation.commit_id(),
            version: observation.version_id(),
        }
    }
}

pub(super) fn transaction_error<'a>(
    denial: PublicationPreparationDenial,
    basis: impl Into<PreparationFailureBasis<'a>>,
    commit_log: CommitLog,
) -> TransactionCommitError {
    let error = match denial {
        PublicationPreparationDenial::Execution(cause) => {
            TransactionCommitError::execution(crate::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenial {
                kind: crate::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenialKind::Cause(cause),
                partition_identity: None,
            })
        }
        PublicationPreparationDenial::StorageUnavailable(failure) => {
            let basis = basis.into();
            TransactionCommitError::preparation(CommitPreparationError::storage_unavailable(
                basis.branch_id.clone(),
                basis.commit_id.map(|id| id.0),
                basis.version,
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
    use crate::branch::root::RelationalBranchRootCaptureDenial;
    use crate::storage::data::{
        RelationalStorageSubject, RelationalStorageUnavailable, RelationalStorageUnavailableCause,
    };
    use crate::transactions::data::outcomes::commit_preparation::CommitPreparationReason;

    #[test]
    fn root_storage_refusal_preserves_its_subject_cause_and_original_branch_basis() {
        let original_branch = BranchId("typed-root-refusal".to_owned());
        let original_commit = CommitId(27);
        let original_version = VersionId(35);
        let basis = PreparationFailureBasis {
            branch_id: &original_branch,
            commit_id: Some(original_commit),
            version: original_version,
        };
        let failure = RelationalStorageUnavailable {
            subject: RelationalStorageSubject::Region(crate::identity::data::PartitionId::main()),
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
        assert_eq!(error.branch_id(), &original_branch);
        assert_eq!(error.expected_commit_id(), Some(original_commit.0));
        assert_eq!(error.expected_version_id(), original_version);
    }
}
