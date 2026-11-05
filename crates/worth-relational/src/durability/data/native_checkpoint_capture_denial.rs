use worth_execution::ExecutionAllocationDenial;

use super::DurabilityError;

/// Runtime capture refusal. Allocation custody is not a durable recovery record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RelationalNativeCheckpointCaptureDenial {
    AdmissionImage(crate::durability::checkpoint_admission::RelationalCheckpointAdmissionRefusal),
    Durability(DurabilityError),
    OwnerUnavailable,
    OriginalAuthorityUnavailable,
    ArtifactFrame(
        crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactFrameRefusal,
    ),
    Content(crate::content::RelationalContentDenial),
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
            Self::AdmissionImage(cause) => {
                write!(formatter, "native checkpoint admission image: {cause:?}")
            }
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
            Self::ArtifactFrame(cause) => write!(
                formatter,
                "native checkpoint source artifact frame: {cause:?}"
            ),
            Self::Content(cause) => write!(formatter, "native checkpoint content: {cause:?}"),
            Self::Durability(error) => write!(formatter, "native checkpoint durability: {error:?}"),
            Self::Request(cause) => write!(formatter, "native checkpoint request: {cause:?}"),
            Self::Allocation(error) => write!(formatter, "native checkpoint allocation: {error}"),
        }
    }
}

impl std::error::Error for RelationalNativeCheckpointCaptureDenial {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ArtifactFrame(cause) => Some(cause),
            Self::Allocation(error) => Some(error),
            Self::Content(_)
            | Self::AdmissionImage(_)
            | Self::Durability(_)
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
            Encode::Frame(cause) => Self::ArtifactFrame(cause),
            Encode::Request(cause) => Self::Request(cause),
            Encode::Allocation(error) => Self::Allocation(error),
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
            Failure::Content(cause) => cause.into(),
            Failure::StorageUnavailable(failure) => failure.into(),
            Failure::Request(cause) => Self::Request(cause),
        }
    }
}

impl From<crate::content::RelationalContentDenial> for RelationalNativeCheckpointCaptureDenial {
    fn from(cause: crate::content::RelationalContentDenial) -> Self {
        match cause {
            crate::content::RelationalContentDenial::Execution(cause) => Self::Request(cause),
            cause => Self::Content(cause),
        }
    }
}
impl From<super::RelationalRecoveryStop> for RelationalNativeCheckpointCaptureDenial {
    fn from(cause: super::RelationalRecoveryStop) -> Self {
        match cause {
            super::RelationalRecoveryStop::AdmissionImage(cause) => Self::AdmissionImage(cause),
            super::RelationalRecoveryStop::Request(cause) => Self::Request(cause),
            super::RelationalRecoveryStop::Allocation(cause) => Self::Allocation(cause),
            super::RelationalRecoveryStop::Durability(cause) => Self::Durability(cause),
            super::RelationalRecoveryStop::Content(cause) => Self::Content(cause),
        }
    }
}
