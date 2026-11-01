use worth_execution::ExecutionAllocationDenial;

use super::DurabilityError;

/// Runtime capture refusal. Allocation custody is not a durable recovery record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RelationalNativeCheckpointCaptureDenial {
    Durability(DurabilityError),
    OwnerUnavailable,
    OriginalAuthorityUnavailable,
    Allocation(ExecutionAllocationDenial),
    Request(crate::execution::RelationalExecutionDenialCause),
    RetentionCapacityExhausted,
    RetentionIdentityExhausted,
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
            Self::RetentionCapacityExhausted => {
                formatter.write_str("native checkpoint retention capacity exhausted")
            }
            Self::RetentionIdentityExhausted => {
                formatter.write_str("native checkpoint retention identity exhausted")
            }
            Self::OwnerUnavailable => formatter.write_str("native checkpoint owner unavailable"),
            Self::OriginalAuthorityUnavailable => {
                formatter.write_str("original complete checkpoint authority unavailable")
            }
            Self::Durability(error) => write!(formatter, "native checkpoint durability: {error:?}"),
            Self::Request(cause) => write!(formatter, "native checkpoint request: {cause:?}"),
            Self::Allocation(error) => write!(formatter, "native checkpoint allocation: {error}"),
        }
    }
}

impl std::error::Error for RelationalNativeCheckpointCaptureDenial {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Allocation(error) => Some(error),
            Self::Durability(_)
            | Self::Request(_)
            | Self::OwnerUnavailable
            | Self::OriginalAuthorityUnavailable
            | Self::RetentionCapacityExhausted
            | Self::RetentionIdentityExhausted => None,
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
            Encode::Request(cause) => Self::Request(cause.into()),
            Encode::Readmission(cause) => {
                crate::durability::checkpoint_history::capture::capture_stop(cause)
            }
            Encode::StorageUnavailable(failure) => Self::Durability(failure.into()),
            error => Self::Durability(super::DurabilityError::new(
                super::RecoveryFailureClass::CorruptCheckpoint,
                format!("inline canonical artifact encoding: {error}"),
            )),
        }
    }
}

impl From<crate::storage::data::RelationalStorageUnavailable>
    for RelationalNativeCheckpointCaptureDenial
{
    fn from(failure: crate::storage::data::RelationalStorageUnavailable) -> Self {
        Self::Durability(failure.into())
    }
}

impl From<crate::storage::residency::RelationalResidencyDemandFailure>
    for RelationalNativeCheckpointCaptureDenial
{
    fn from(failure: crate::storage::residency::RelationalResidencyDemandFailure) -> Self {
        use crate::storage::residency::RelationalResidencyDemandFailure as Failure;
        match failure {
            Failure::StorageUnavailable(failure) => failure.into(),
            Failure::Request(cause) => Self::Request(cause),
        }
    }
}
