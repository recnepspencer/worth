use std::time::Instant;

use super::{CancellationToken, SerialMemoryBudget};
use worth_foundational::ExecutionRequestPolicy;

/// A reusable description of a lease-free request, built from the original optional memory policy.
/// Memory and work ceilings are optional; computation declarations still constrain the same request.
/// It may outlive a scope; every use opens an accounted scope or draws from its parent.
/// The description grants no access to an absent lease outside that closure.
#[derive(Debug, Clone)]
pub struct SerialRequest {
    pub(crate) memory: SerialMemoryBudget,
    pub(crate) work_ceiling: Option<u64>,
    pub(crate) deadline: Option<Instant>,
    pub(crate) cancellation: CancellationToken,
}

impl SerialRequest {
    pub(crate) fn stop(&self) -> Option<crate::backend::KernelStop> {
        use crate::backend::KernelStop;
        if self.cancellation.is_cancelled() {
            Some(KernelStop::Cancelled)
        } else if self
            .deadline
            .is_some_and(|deadline| Instant::now() >= deadline)
        {
            Some(KernelStop::DeadlineElapsed)
        } else {
            None
        }
    }

    pub fn from_policy(
        policy: &ExecutionRequestPolicy,
        cancellation: CancellationToken,
        deadline: Option<Instant>,
    ) -> Self {
        Self {
            memory: SerialMemoryBudget::from_policy(policy),
            work_ceiling: policy.budget().work_ceiling(),
            cancellation,
            deadline,
        }
    }

    /// The budget carries the owner's policy memory limit.
    pub fn from_memory(
        memory: SerialMemoryBudget,
        cancellation: CancellationToken,
        deadline: Option<Instant>,
    ) -> Self {
        Self {
            memory,
            work_ceiling: None,
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
