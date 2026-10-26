use worth_execution::ExecutionAllocationDenial;

use super::DurabilityError;

/// Runtime capture refusal. Allocation custody is not a durable recovery record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RelationalNativeCheckpointCaptureDenial {
    Durability(DurabilityError),
    Allocation(ExecutionAllocationDenial),
}

impl From<DurabilityError> for RelationalNativeCheckpointCaptureDenial {
    fn from(error: DurabilityError) -> Self {
        Self::Durability(error)
    }
}

impl From<ExecutionAllocationDenial> for RelationalNativeCheckpointCaptureDenial {
    fn from(error: ExecutionAllocationDenial) -> Self {
        Self::Allocation(error)
    }
}

impl std::fmt::Display for RelationalNativeCheckpointCaptureDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Durability(error) => write!(formatter, "native checkpoint durability: {error:?}"),
            Self::Allocation(error) => write!(formatter, "native checkpoint allocation: {error}"),
        }
    }
}

impl std::error::Error for RelationalNativeCheckpointCaptureDenial {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Allocation(error) => Some(error),
            Self::Durability(_) => None,
        }
    }
}

impl
    From<
        crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactEncodeDenial,
    > for RelationalNativeCheckpointCaptureDenial
{
    fn from(
        error: crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactEncodeDenial,
    ) -> Self {
        use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactEncodeDenial as Encode;
        match error {
            Encode::Allocation(error) => Self::Allocation(error),
            Encode::StorageUnavailable(failure) => {
                let super::RelationalRecoveryStop::Durability(error) = failure.into() else {
                    unreachable!()
                };
                Self::Durability(error)
            }
            error => Self::Durability(super::DurabilityError::new(
                super::RecoveryFailureClass::CorruptCheckpoint,
                format!("inline canonical artifact encoding: {error}"),
            )),
        }
    }
}
