use worth_execution::ExecutionAllocationDenial;
use worth_relational::facade::durability::{
    DurabilityError, RelationalNativeCheckpointCaptureDenial,
};

/// Exact refusal from native checkpoint capture, framing or final byte allocation.
/// An allocation refusal never authorizes a different capture policy.
#[derive(Debug)]
pub enum WorthQueryCheckpointCaptureDenial {
    ForeignPhase(crate::domain_computation::request_execution::WorthQueryForeignAdvancementPhase),
    /// Boxed: the native durability refusal is the largest cause, and every
    /// capture result would otherwise reserve its bytes inline.
    Durability(Box<DurabilityError>),
    Content(worth_relational::facade::content::RelationalContentDenial),
    ArtifactFrame(
        worth_relational::facade::durability::RelationalCanonicalCommitArtifactFrameRefusal,
    ),
    Allocation(ExecutionAllocationDenial),
    /// Original execution stopped, or cannot fund the required capture contract.
    Execution(
        crate::domain_computation::graph_layout::denial::WorthQueryPrimaryGraphInstallationDenial,
    ),
}

impl From<RelationalNativeCheckpointCaptureDenial> for WorthQueryCheckpointCaptureDenial {
    fn from(error: RelationalNativeCheckpointCaptureDenial) -> Self {
        match error {
            RelationalNativeCheckpointCaptureDenial::Content(cause) => Self::Content(cause),
            RelationalNativeCheckpointCaptureDenial::ArtifactFrame(cause) => {
                Self::ArtifactFrame(cause)
            }
            RelationalNativeCheckpointCaptureDenial::Durability(error) => error.into(),
            RelationalNativeCheckpointCaptureDenial::Allocation(error) => Self::Allocation(error),
            RelationalNativeCheckpointCaptureDenial::Request(cause) => {
                use crate::domain_computation::graph_layout::denial::WorthQueryPrimaryGraphInstallationDenialKind as Kind;
                let kind = match crate::domain_computation::primary_graph::vocabulary::provider::relational_execution_denial::relational_execution_kind(cause, None) {
                    Ok(kind) => Kind::ExecutionDenied { kind },
                    Err(kind) => Kind::ExecutionControlStopped { kind },
                };
                Self::Execution(crate::domain_computation::graph_layout::denial::WorthQueryPrimaryGraphInstallationDenial::new(
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

impl From<crate::domain_computation::request_execution::WorthQueryForeignAdvancementPhase>
    for WorthQueryCheckpointCaptureDenial
{
    fn from(
        error: crate::domain_computation::request_execution::WorthQueryForeignAdvancementPhase,
    ) -> Self {
        Self::ForeignPhase(error)
    }
}
