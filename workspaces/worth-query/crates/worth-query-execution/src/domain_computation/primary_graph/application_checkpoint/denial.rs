use worth_execution::ExecutionAllocationDenial;
use worth_relational::facade::durability::{
    DurabilityError, RelationalNativeCheckpointCaptureDenial,
};

/// Exact refusal from native checkpoint capture, framing or final byte allocation.
/// An allocation refusal never authorizes a different capture policy.
#[derive(Debug)]
pub enum WorthQueryCheckpointCaptureDenial {
    WorldOriginals(Box<worth_runtime_world::facade::WorldInterimCheckpointRefusal>),
    AdmissionImage(Box<worth_relational::facade::durability::RelationalCheckpointAdmissionRefusal>),
    ForeignPhase(super::super::WorthQueryForeignAdvancementPhase),
    NativeOwnerUnavailable,
    OriginalAuthorityUnavailable,
    RetentionCapacityExhausted,
    RetentionIdentityExhausted,
    /// Boxed: the native durability refusal is the largest cause, and every
    /// capture result would otherwise reserve its bytes inline.
    Durability(Box<DurabilityError>),
    Content(worth_relational::facade::content::RelationalContentDenial),
    ArtifactFrame(
        worth_relational::facade::durability::RelationalCanonicalCommitArtifactFrameRefusal,
    ),
    Allocation(ExecutionAllocationDenial),
    /// Original execution stopped, or cannot fund the required capture contract.
    Execution(super::super::WorthQueryPrimaryGraphInstallationDenial),
}

impl From<RelationalNativeCheckpointCaptureDenial> for WorthQueryCheckpointCaptureDenial {
    fn from(error: RelationalNativeCheckpointCaptureDenial) -> Self {
        match error {
            RelationalNativeCheckpointCaptureDenial::AdmissionImage(error) => {
                Self::AdmissionImage(Box::new(error))
            }
            RelationalNativeCheckpointCaptureDenial::Content(cause) => Self::Content(cause),
            RelationalNativeCheckpointCaptureDenial::ArtifactFrame(cause) => {
                Self::ArtifactFrame(cause)
            }
            RelationalNativeCheckpointCaptureDenial::Durability(error) => error.into(),
            RelationalNativeCheckpointCaptureDenial::OwnerUnavailable => {
                Self::NativeOwnerUnavailable
            }
            RelationalNativeCheckpointCaptureDenial::OriginalAuthorityUnavailable => {
                Self::OriginalAuthorityUnavailable
            }
            RelationalNativeCheckpointCaptureDenial::RetentionCapacityExhausted => {
                Self::RetentionCapacityExhausted
            }
            RelationalNativeCheckpointCaptureDenial::RetentionIdentityExhausted => {
                Self::RetentionIdentityExhausted
            }
            RelationalNativeCheckpointCaptureDenial::Allocation(error) => Self::Allocation(error),
            RelationalNativeCheckpointCaptureDenial::Request(cause) => {
                use super::super::WorthQueryPrimaryGraphInstallationDenialKind as Kind;
                let kind = match super::super::provider::relational_execution_denial::relational_execution_kind(cause, None) {
                    Ok(kind) => Kind::ExecutionDenied { kind },
                    Err(kind) => Kind::ExecutionControlStopped { kind },
                };
                Self::Execution(super::super::WorthQueryPrimaryGraphInstallationDenial::new(
                    kind,
                    "checkpoint history consolidation stopped",
                ))
            }
        }
    }
}

impl From<DurabilityError> for WorthQueryCheckpointCaptureDenial {
    fn from(error: DurabilityError) -> Self {
        Self::Durability(Box::new(error))
    }
}

impl From<ExecutionAllocationDenial> for WorthQueryCheckpointCaptureDenial {
    fn from(error: ExecutionAllocationDenial) -> Self {
        Self::Allocation(error)
    }
}

impl std::fmt::Display for WorthQueryCheckpointCaptureDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WorldOriginals(error) => {
                write!(formatter, "checkpoint World originals: {error:?}")
            }
            Self::AdmissionImage(error) => {
                write!(formatter, "checkpoint Native admission image: {error:?}")
            }
            Self::NativeOwnerUnavailable => {
                formatter.write_str("checkpoint Native owner unavailable")
            }
            Self::OriginalAuthorityUnavailable => {
                formatter.write_str("checkpoint original authority unavailable")
            }
            Self::RetentionCapacityExhausted => {
                formatter.write_str("checkpoint retention capacity exhausted")
            }
            Self::RetentionIdentityExhausted => {
                formatter.write_str("checkpoint retention identity exhausted")
            }
            Self::Content(cause) => write!(formatter, "checkpoint content: {cause:?}"),
            Self::ArtifactFrame(cause) => write!(formatter, "checkpoint artifact frame: {cause:?}"),
            Self::Execution(error) => write!(formatter, "checkpoint execution: {error:?}"),
            Self::ForeignPhase(error) => write!(formatter, "checkpoint phase: {error:?}"),
            Self::Durability(error) => write!(formatter, "checkpoint durability: {error:?}"),
            Self::Allocation(error) => write!(formatter, "checkpoint allocation: {error:?}"),
        }
    }
}

impl std::error::Error for WorthQueryCheckpointCaptureDenial {}

impl From<super::super::WorthQueryForeignAdvancementPhase> for WorthQueryCheckpointCaptureDenial {
    fn from(error: super::super::WorthQueryForeignAdvancementPhase) -> Self {
        Self::ForeignPhase(error)
    }
}
