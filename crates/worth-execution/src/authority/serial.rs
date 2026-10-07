use std::time::Instant;

use super::{CancellationToken, SerialMemoryBudget};
use worth_foundational::ExecutionRequestPolicy;

/// A reusable description of a lease-free request, built from policy-bounded memory.
/// Serial requests have no request-level work ceiling; computation declarations still bind.
/// It may outlive a scope; every use opens a bounded scope or draws from its parent.
/// The description grants no access to an absent lease outside that closure.
#[derive(Debug, Clone)]
pub struct SerialRequest {
    pub(crate) memory: SerialMemoryBudget,
    pub(crate) deadline: Option<Instant>,
    pub(crate) cancellation: CancellationToken,
}

impl SerialRequest {
    pub fn from_policy(
        policy: &ExecutionRequestPolicy,
        cancellation: CancellationToken,
        deadline: Option<Instant>,
    ) -> Self {
        Self::from_memory(
            SerialMemoryBudget::from_policy(policy),
            cancellation,
            deadline,
        )
    }

    /// The budget carries the owner's policy memory limit.
    pub fn from_memory(
        memory: SerialMemoryBudget,
        cancellation: CancellationToken,
        deadline: Option<Instant>,
    ) -> Self {
        Self {
            memory,
            cancellation,
            deadline,
        }
    }

    pub fn with_cancellation(mut self, cancellation: CancellationToken) -> Self {
        self.cancellation = cancellation;
        self
    }
    pub fn with_deadline(mut self, deadline: Option<Instant>) -> Self {
        self.deadline = deadline;
        self
    }

    pub fn memory(&self) -> &SerialMemoryBudget {
        &self.memory
    }
}
