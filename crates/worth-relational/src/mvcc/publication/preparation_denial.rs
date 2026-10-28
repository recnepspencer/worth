//! Preserve storage refusals across pre-effect publication preparation.
use crate::branch::RelationalBranchRootCaptureDenial;
use crate::storage::data::RelationalStorageUnavailable;

#[derive(Debug)]
pub(crate) enum PublicationPreparationDenial {
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
            RelationalBranchRootCaptureDenial::StorageUnavailable(failure) => {
                Self::StorageUnavailable(failure)
            }
            other => Self::Invalid(format!("branch-root capture denied: {other:?}")),
        }
    }
}
