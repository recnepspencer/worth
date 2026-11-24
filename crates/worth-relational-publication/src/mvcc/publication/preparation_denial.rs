//! Preserve storage refusals across pre-effect publication preparation.
use contracts::branch::root_capture_denial::RelationalBranchRootCaptureDenial;
use contracts::storage::data::storage_unavailable::RelationalStorageUnavailable;
use worth_relational_contracts::internal_port as contracts;

#[derive(Debug)]
pub enum PublicationPreparationDenial {
    Execution(contracts::execution::denial_cause::RelationalExecutionDenialCause),
    Invalid(String),
    StorageUnavailable(RelationalStorageUnavailable),
}

impl From<String> for PublicationPreparationDenial {
    fn from(detail: String) -> Self {
        Self::Invalid(detail)
    }
}

impl From<RelationalBranchRootCaptureDenial> for PublicationPreparationDenial {
    fn from(denial: RelationalBranchRootCaptureDenial) -> Self {
        match denial {
            RelationalBranchRootCaptureDenial::Execution(cause)
            | RelationalBranchRootCaptureDenial::Request(cause) => Self::Execution(cause),
            RelationalBranchRootCaptureDenial::StorageUnavailable(failure) => {
                Self::StorageUnavailable(failure)
            }
            other => Self::Invalid(format!("branch-root capture denied: {other:?}")),
        }
    }
}
