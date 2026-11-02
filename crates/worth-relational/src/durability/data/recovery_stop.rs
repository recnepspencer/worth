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
    Request(RelationalExecutionDenialCause),
    Allocation(worth_execution::ExecutionAllocationDenial),
    Durability(DurabilityError),
    Content(crate::content::RelationalContentDenial),
}

impl From<RelationalExecutionDenialCause> for RelationalRecoveryStop {
    fn from(cause: RelationalExecutionDenialCause) -> Self {
        match cause {
            RelationalExecutionDenialCause::StorageUnavailable(failure) => failure.into(),
            cause => Self::Request(cause),
        }
    }
}

impl From<DurabilityError> for RelationalRecoveryStop {
    fn from(error: DurabilityError) -> Self {
        Self::Durability(error)
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
            super::RelationalNativeCheckpointCaptureDenial::ArtifactFrame(cause) => {
                use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactFrameRefusal as Frame;
                let class = match cause {
                    Frame::UnsupportedVersion { .. } => {
                        super::RecoveryFailureClass::UnsupportedLegacySemantics
                    }
                    Frame::SizeLimit { .. } => super::RecoveryFailureClass::CheckpointSizeOverflow,
                    Frame::UnrecognizedMagic
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
