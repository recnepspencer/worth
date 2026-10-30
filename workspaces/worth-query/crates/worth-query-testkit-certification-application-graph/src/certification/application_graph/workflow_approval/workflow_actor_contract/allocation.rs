//! The genuine committed actor footprint must be paid when retained again.
pub use super::*;
pub use std::num::NonZeroUsize;
pub use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
pub use worth_query_host::facade::runtime::{CancellationToken, LeaseRequest};

pub fn request(memory: u64) -> LeaseRequest {
    LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::new(1).unwrap(), Some(memory), Some(1)),
        ),
        deadline: None,
        cancellation: CancellationToken::new(),
    }
}
