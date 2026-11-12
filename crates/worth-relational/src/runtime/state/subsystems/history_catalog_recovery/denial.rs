//! Reconstruction failures preserve original control and Native content causes.
use crate::branch::RelationalBranchRootCaptureDenial;
use crate::durability::data::{DurabilityError, RecoveryFailureClass, RelationalRecoveryStop};

#[derive(Debug)]
pub(crate) enum RelationalHistoryRecoveryDenial {
    Invalid(String),
    Root(RelationalBranchRootCaptureDenial),
    Source(RelationalRecoveryStop),
    Artifact(crate::history::RelationalCommitArtifactDenial),
    Catalog(crate::history::RelationalCommitCatalogAppendDenial),
}
impl From<String> for RelationalHistoryRecoveryDenial {
    fn from(detail: String) -> Self {
        Self::Invalid(detail)
    }
}
impl From<RelationalBranchRootCaptureDenial> for RelationalHistoryRecoveryDenial {
    fn from(cause: RelationalBranchRootCaptureDenial) -> Self {
        Self::Root(cause)
    }
}
impl From<RelationalHistoryRecoveryDenial> for RelationalRecoveryStop {
    fn from(cause: RelationalHistoryRecoveryDenial) -> Self {
        match cause {
            RelationalHistoryRecoveryDenial::Source(cause) => cause,
            RelationalHistoryRecoveryDenial::Root(RelationalBranchRootCaptureDenial::Content(
                cause,
            )) => cause.into(),
            RelationalHistoryRecoveryDenial::Root(
                RelationalBranchRootCaptureDenial::Request(cause)
                | RelationalBranchRootCaptureDenial::Execution(cause),
            ) => RelationalRecoveryStop::Request(cause),
            cause => {
                DurabilityError::new(RecoveryFailureClass::CorruptCheckpoint, cause.to_string())
                    .into()
            }
        }
    }
}
impl std::fmt::Display for RelationalHistoryRecoveryDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(detail) => f.write_str(detail),
            Self::Root(cause) => write!(f, "root readmission refused: {cause:?}"),
            Self::Source(cause) => write!(f, "source readmission refused: {cause:?}"),
            Self::Artifact(cause) => write!(f, "artifact preparation refused: {cause:?}"),
            Self::Catalog(cause) => write!(f, "catalog replacement refused: {cause:?}"),
        }
    }
}
impl std::error::Error for RelationalHistoryRecoveryDenial {}
