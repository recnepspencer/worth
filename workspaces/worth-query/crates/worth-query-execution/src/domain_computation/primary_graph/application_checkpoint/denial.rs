use worth_execution::ExecutionAllocationDenial;
use worth_relational::facade::durability::{
    DurabilityError, RelationalNativeCheckpointCaptureDenial,
};

/// Exact refusal from native checkpoint capture, framing or final byte allocation.
/// An allocation refusal never authorizes a different capture policy.
#[derive(Debug)]
pub enum WorthQueryCheckpointCaptureDenial {
    Durability(DurabilityError),
    Allocation(ExecutionAllocationDenial),
}

impl From<RelationalNativeCheckpointCaptureDenial> for WorthQueryCheckpointCaptureDenial {
    fn from(error: RelationalNativeCheckpointCaptureDenial) -> Self {
        match error {
            RelationalNativeCheckpointCaptureDenial::Durability(error) => Self::Durability(error),
            RelationalNativeCheckpointCaptureDenial::Allocation(error) => Self::Allocation(error),
        }
    }
}

impl From<DurabilityError> for WorthQueryCheckpointCaptureDenial {
    fn from(error: DurabilityError) -> Self {
        Self::Durability(error)
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
            Self::Durability(error) => write!(formatter, "checkpoint durability: {error:?}"),
            Self::Allocation(error) => write!(formatter, "checkpoint allocation: {error:?}"),
        }
    }
}

impl std::error::Error for WorthQueryCheckpointCaptureDenial {}
