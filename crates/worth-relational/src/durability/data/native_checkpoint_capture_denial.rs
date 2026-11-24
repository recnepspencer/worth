use super::recovery_errors::{DurabilityError, RecoveryFailureClass};
use super::recovery_stop::RelationalRecoveryStop;
use worth_execution::ExecutionAllocationDenial;

/// Runtime capture refusal. Allocation custody is not a durable recovery record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RelationalNativeCheckpointCaptureDenial {
    AdmissionImage(crate::durability::checkpoint_admission::refusal::RelationalCheckpointAdmissionRefusal),
    Durability(DurabilityError),
    OwnerUnavailable,
    OriginalAuthorityUnavailable,
    ArtifactFrame(
        crate::durability::canonical_commit_artifact::frame_refusal::RelationalCanonicalCommitArtifactFrameRefusal,
    ),
    Content(crate::content::denial::RelationalContentDenial),
    Allocation(ExecutionAllocationDenial),
    Request(crate::execution::denial_cause::RelationalExecutionDenialCause),
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

impl From<crate::storage::data::storage_unavailable::RelationalStorageUnavailable>
    for RelationalNativeCheckpointCaptureDenial
{
    fn from(
        failure: crate::storage::data::storage_unavailable::RelationalStorageUnavailable,
    ) -> Self {
        Self::Durability(failure.into())
    }
}

impl From<crate::content::denial::RelationalContentDenial>
    for RelationalNativeCheckpointCaptureDenial
{
    fn from(cause: crate::content::denial::RelationalContentDenial) -> Self {
        match cause {
            crate::content::denial::RelationalContentDenial::Execution(cause) => {
                Self::Request(cause)
            }
            cause => Self::Content(cause),
        }
    }
}
impl From<RelationalRecoveryStop> for RelationalNativeCheckpointCaptureDenial {
    fn from(cause: RelationalRecoveryStop) -> Self {
        match cause {
            RelationalRecoveryStop::AdmissionImage(cause) => Self::AdmissionImage(cause),
            RelationalRecoveryStop::Request(cause) => Self::Request(cause),
            RelationalRecoveryStop::Allocation(cause) => Self::Allocation(cause),
            RelationalRecoveryStop::Durability(cause) => Self::Durability(cause),
            RelationalRecoveryStop::Content(cause) => Self::Content(cause),
        }
    }
}

impl From<RelationalNativeCheckpointCaptureDenial> for RelationalRecoveryStop {
    fn from(denial: RelationalNativeCheckpointCaptureDenial) -> Self {
        match denial {
            RelationalNativeCheckpointCaptureDenial::AdmissionImage(cause) => cause.into(),
            RelationalNativeCheckpointCaptureDenial::RetentionCapacityExhausted => {
                DurabilityError::new(
                    RecoveryFailureClass::ColdStorageUnavailable,
                    "checkpoint retention capacity exhausted",
                )
                .into()
            }
            RelationalNativeCheckpointCaptureDenial::RetentionIdentityExhausted => {
                DurabilityError::new(
                    RecoveryFailureClass::ColdStorageUnavailable,
                    "checkpoint retention identity exhausted",
                )
                .into()
            }
            RelationalNativeCheckpointCaptureDenial::OwnerUnavailable => DurabilityError::new(
                RecoveryFailureClass::ColdStorageUnavailable,
                "checkpoint owner unavailable",
            )
            .into(),
            RelationalNativeCheckpointCaptureDenial::OriginalAuthorityUnavailable => {
                DurabilityError::new(
                    RecoveryFailureClass::UnsupportedLegacySemantics,
                    "original complete checkpoint authority unavailable",
                )
                .into()
            }
            RelationalNativeCheckpointCaptureDenial::ArtifactFrame(cause) => {
                use crate::durability::canonical_commit_artifact::frame_refusal::RelationalCanonicalCommitArtifactFrameRefusal as Frame;
                let class = match cause {
                    Frame::UnsupportedVersion(_) => {
                        RecoveryFailureClass::UnsupportedLegacySemantics
                    }
                    Frame::SizeLimit { .. } => RecoveryFailureClass::CheckpointSizeOverflow,
                    Frame::InvalidVersion
                    | Frame::InvalidPayloadShape
                    | Frame::UnrecognizedMagic
                    | Frame::TruncatedHeader { .. }
                    | Frame::TruncatedPayload { .. }
                    | Frame::TrailingBytes { .. }
                    | Frame::DigestMismatch { .. } => RecoveryFailureClass::CorruptCheckpoint,
                };
                Self::Durability(DurabilityError::new(
                    class,
                    format!("checkpoint source artifact frame: {cause:?}"),
                ))
            }
            RelationalNativeCheckpointCaptureDenial::Content(cause) => Self::from(cause),
            RelationalNativeCheckpointCaptureDenial::Request(cause) => Self::Request(cause),
            RelationalNativeCheckpointCaptureDenial::Durability(error) => error.into(),
            RelationalNativeCheckpointCaptureDenial::Allocation(error) => {
                crate::symbols::data::field_text_recovery_stop(
                    crate::symbols::data::RelationalFieldTextAdmissionDenial::Allocation(error),
                )
            }
        }
    }
}

pub(crate) fn capture_stop(
    stop: RelationalRecoveryStop,
) -> RelationalNativeCheckpointCaptureDenial {
    match stop {
        RelationalRecoveryStop::Content(cause) => cause.into(),
        RelationalRecoveryStop::AdmissionImage(cause) => {
            RelationalNativeCheckpointCaptureDenial::AdmissionImage(cause)
        }
        RelationalRecoveryStop::Request(cause) => {
            RelationalNativeCheckpointCaptureDenial::Request(cause)
        }
        RelationalRecoveryStop::Allocation(cause) => {
            RelationalNativeCheckpointCaptureDenial::Allocation(cause)
        }
        RelationalRecoveryStop::Durability(cause) => {
            RelationalNativeCheckpointCaptureDenial::Durability(cause)
        }
    }
}
pub(crate) fn array_capture_stop(
    cause: worth_execution::ExecutionRequestArrayDenial,
) -> RelationalNativeCheckpointCaptureDenial {
    match cause {
        worth_execution::ExecutionRequestArrayDenial::Request(cause) => {
            RelationalNativeCheckpointCaptureDenial::Request(cause.into())
        }
        worth_execution::ExecutionRequestArrayDenial::Allocation(cause) => {
            RelationalNativeCheckpointCaptureDenial::Allocation(cause)
        }
    }
}
