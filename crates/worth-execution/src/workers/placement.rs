use core::num::NonZeroUsize;

use worth_foundational::{DeterminismContract, ExecutionPosture, ExecutionRequestPolicy};

/// Placement for independent partitions. The existing Execution authority
/// admits the worker bound and retains canonical settlement order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutionPlacement {
    Serial,
    Parallel { max_workers: NonZeroUsize },
}

impl ExecutionPlacement {
    /// Builds a worker-only policy for [`crate::ExecutionAuthority::request_lease`].
    /// Parallel policy may resolve serially on a serial target or at width one.
    pub const fn worker_policy(self, determinism: DeterminismContract) -> ExecutionRequestPolicy {
        let (posture, width) = match self {
            Self::Serial => (ExecutionPosture::Serial, NonZeroUsize::MIN),
            Self::Parallel { max_workers } => (ExecutionPosture::Automatic, max_workers),
        };
        ExecutionRequestPolicy::for_workers(posture, determinism, width)
    }
}
