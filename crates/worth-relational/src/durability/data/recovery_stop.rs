use crate::execution::RelationalExecutionDenialCause;

use super::DurabilityError;

/// An in-process recovery refusal. Request control is never durable evidence.
///
/// Durable failures retain their existing serialization contract:
/// ```
/// fn durable<T: serde::Serialize>() {}
/// durable::<worth_relational::facade::durability::DurabilityError>();
/// ```
/// A recovery stop cannot be persisted as a recovery failure:
/// ```compile_fail,E0277
/// fn durable<T: serde::Serialize>() {}
/// durable::<worth_relational::facade::durability::RelationalRecoveryStop>();
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationalRecoveryStop {
    AdmissionImage(crate::durability::checkpoint_admission::RelationalCheckpointAdmissionRefusal),
    Request(RelationalExecutionDenialCause),
    Allocation(worth_execution::ExecutionAllocationDenial),
    Durability(DurabilityError),
    Content(crate::content::RelationalContentDenial),
}

impl From<RelationalExecutionDenialCause> for RelationalRecoveryStop {
    fn from(cause: RelationalExecutionDenialCause) -> Self {
        match cause {
            RelationalExecutionDenialCause::StorageUnavailable(failure) => failure.into(),
            RelationalExecutionDenialCause::Content(cause) => Self::Content(cause.into()),
            cause => Self::Request(cause),
        }
    }
}

impl From<DurabilityError> for RelationalRecoveryStop {
    fn from(error: DurabilityError) -> Self {
        Self::Durability(error)
    }
}

impl From<crate::durability::checkpoint_admission::RelationalCheckpointAdmissionRefusal>
    for RelationalRecoveryStop
{
    fn from(
        cause: crate::durability::checkpoint_admission::RelationalCheckpointAdmissionRefusal,
    ) -> Self {
        use crate::durability::checkpoint_admission::RelationalCheckpointAdmissionRefusal as Image;
        match cause {
            Image::Request(cause) => Self::Request(cause),
            Image::Content(cause) => cause.into(),
            Image::Allocation(cause) => Self::Allocation(cause),
            Image::Native(cause) => Self::Durability(cause),
            Image::Buffer(worth_execution::ExecutionRequestArrayDenial::Request(cause)) => {
                Self::Request(cause.into())
            }
            Image::Buffer(worth_execution::ExecutionRequestArrayDenial::Allocation(cause)) => {
                Self::Allocation(cause)
            }
            cause @ (Image::MissingArtifact
            | Image::PredecessorFormat { .. }
            | Image::UnsupportedVersion(_)
            | Image::InvalidVersion
            | Image::Malformed
            | Image::BindingMismatch(_)
            | Image::AuthenticationMismatch(_)
            | Image::MissingRootImage
            | Image::InvalidProjectedMeaning
            | Image::MissingCaptureOrigin
            | Image::OwnerUnavailable
            | Image::CapacityOverflow) => Self::AdmissionImage(cause),
        }
    }
}

impl From<crate::storage::data::RelationalStorageUnavailable> for RelationalRecoveryStop {
    fn from(failure: crate::storage::data::RelationalStorageUnavailable) -> Self {
        Self::Durability(failure.into())
    }
}

impl std::fmt::Display for RelationalRecoveryStop {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AdmissionImage(cause) => {
                write!(formatter, "checkpoint admission image: {cause:?}")
            }
            Self::Allocation(cause) => write!(formatter, "recovery allocation stopped: {cause}"),
            Self::Request(cause) => write!(formatter, "recovery request stopped: {cause:?}"),
            Self::Content(cause) => write!(formatter, "recovery content refused: {cause:?}"),
            Self::Durability(error) => write!(formatter, "recovery durability: {error:?}"),
        }
    }
}

impl std::error::Error for RelationalRecoveryStop {}

impl From<super::RelationalNativeCheckpointCaptureDenial> for RelationalRecoveryStop {
    fn from(denial: super::RelationalNativeCheckpointCaptureDenial) -> Self {
        match denial {
            super::RelationalNativeCheckpointCaptureDenial::AdmissionImage(cause) => cause.into(),
            super::RelationalNativeCheckpointCaptureDenial::RetentionCapacityExhausted => {
                super::DurabilityError::new(
                    super::RecoveryFailureClass::ColdStorageUnavailable,
                    "checkpoint retention capacity exhausted",
                )
                .into()
            }
            super::RelationalNativeCheckpointCaptureDenial::RetentionIdentityExhausted => {
                super::DurabilityError::new(
                    super::RecoveryFailureClass::ColdStorageUnavailable,
                    "checkpoint retention identity exhausted",
                )
                .into()
            }
            super::RelationalNativeCheckpointCaptureDenial::OwnerUnavailable => {
                super::DurabilityError::new(
                    super::RecoveryFailureClass::ColdStorageUnavailable,
                    "checkpoint owner unavailable",
                )
                .into()
            }
            super::RelationalNativeCheckpointCaptureDenial::OriginalAuthorityUnavailable => {
                super::DurabilityError::new(
                    super::RecoveryFailureClass::UnsupportedLegacySemantics,
                    "original complete checkpoint authority unavailable",
                )
                .into()
            }
            super::RelationalNativeCheckpointCaptureDenial::ArtifactFrame(cause) => {
                use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactFrameRefusal as Frame;
                let class = match cause {
                    Frame::UnsupportedVersion(_) => {
                        super::RecoveryFailureClass::UnsupportedLegacySemantics
                    }
                    Frame::SizeLimit { .. } => super::RecoveryFailureClass::CheckpointSizeOverflow,
                    Frame::InvalidVersion
                    | Frame::InvalidPayloadShape
                    | Frame::UnrecognizedMagic
                    | Frame::TruncatedHeader { .. }
                    | Frame::TruncatedPayload { .. }
                    | Frame::TrailingBytes { .. }
                    | Frame::DigestMismatch { .. } => {
                        super::RecoveryFailureClass::CorruptCheckpoint
                    }
                };
                Self::Durability(DurabilityError::new(
                    class,
                    format!("checkpoint source artifact frame: {cause:?}"),
                ))
            }
            super::RelationalNativeCheckpointCaptureDenial::Content(cause) => Self::from(cause),
            super::RelationalNativeCheckpointCaptureDenial::Request(cause) => Self::Request(cause),
            super::RelationalNativeCheckpointCaptureDenial::Durability(error) => error.into(),
            super::RelationalNativeCheckpointCaptureDenial::Allocation(error) => {
                crate::symbols::data::field_text_recovery_stop(
                    crate::symbols::data::RelationalFieldTextAdmissionDenial::Allocation(error),
                )
            }
        }
    }
}

impl From<crate::content::RelationalContentDenial> for RelationalRecoveryStop {
    fn from(cause: crate::content::RelationalContentDenial) -> Self {
        match cause {
            crate::content::RelationalContentDenial::Execution(cause) => Self::from(cause),
            cause => Self::Content(cause),
        }
    }
}
