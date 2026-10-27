use worth_execution::ExecutionAllocationDenial;
use worth_relational::facade::durability::{
    DurabilityError, RelationalNativeCheckpointCaptureDenial,
};

/// Exact refusal from native checkpoint capture, framing or final byte allocation.
/// An allocation refusal never authorizes a different capture policy.
#[derive(Debug)]
pub enum WorthQueryCheckpointCaptureDenial {
    ForeignPhase(super::super::WorthQueryForeignAdvancementPhase),
    /// Boxed: the native durability refusal is the largest cause, and every
    /// capture result would otherwise reserve its bytes inline.
    Durability(Box<DurabilityError>),
    Allocation(ExecutionAllocationDenial),
    /// Original execution stopped, or cannot fund the required capture contract.
    Execution(super::super::WorthQueryPrimaryGraphInstallationDenial),
}

impl From<RelationalNativeCheckpointCaptureDenial> for WorthQueryCheckpointCaptureDenial {
    fn from(error: RelationalNativeCheckpointCaptureDenial) -> Self {
        match error {
            RelationalNativeCheckpointCaptureDenial::Durability(error) => error.into(),
            RelationalNativeCheckpointCaptureDenial::Allocation(error) => Self::Allocation(error),
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
