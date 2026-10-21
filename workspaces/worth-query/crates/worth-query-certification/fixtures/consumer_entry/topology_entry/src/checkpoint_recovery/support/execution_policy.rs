use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

// Bounds this checkpoint fixture World's serial request memory and work.
pub(crate) const CHECKPOINT_EXECUTION_POLICY: ExecutionRequestPolicy = ExecutionRequestPolicy::new(
    ExecutionPosture::Serial,
    DeterminismContract::CanonicalBitwise,
    ExecutionBudget::new(std::num::NonZeroUsize::MIN, None, Some(8_000_000)),
);
