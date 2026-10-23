//! Narrow execution posture while retaining the caller's lease lineage.
use worth_execution::{CancellationToken, ExecutionResourceLease, LeaseRequest};
use worth_foundational::{ExecutionPosture, ExecutionRequestPolicy};

pub(super) fn ordered_child<'a>(
    lease: &ExecutionResourceLease<'a>,
) -> Result<ExecutionResourceLease<'a>, crate::execution::PacketExecutionStop> {
    let policy = lease.policy();
    let narrowed = match policy.accounting() {
        Some(budget) => {
            ExecutionRequestPolicy::new(ExecutionPosture::Serial, policy.determinism(), budget)
        }
        None => ExecutionRequestPolicy::for_workers(
            ExecutionPosture::Serial,
            policy.determinism(),
            policy.max_workers(),
        ),
    };
    // The child retains the parent's cancellation, deadline, accounting, and
    // worker limits. Only dispatch order is narrowed; no authority is replaced.
    lease
        .child(LeaseRequest {
            policy: narrowed,
            cancellation: CancellationToken::new(),
            deadline: None,
        })
        .map_err(|denial| crate::execution::PacketExecutionStop::Execution {
            boundary: None,
            reason: worth_execution::MapStop::Admission(denial),
        })
}
