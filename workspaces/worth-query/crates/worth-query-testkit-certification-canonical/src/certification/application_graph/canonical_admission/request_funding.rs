//! Each decoder call owns one bounded caller lease before entering its native phase.
use super::host::{Host, WorthQueryAdvancementPhase, WorthQueryRequestScope};
use worth_execution::{ExecutionAllocationPolicy, ExecutionResourceLease, LeaseRequest};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

pub fn with_request<R>(
    host: &Host,
    scope: &WorthQueryRequestScope,
    body: impl for<'scope> FnOnce(WorthQueryAdvancementPhase<'scope>) -> R,
) -> R {
    let caller = ExecutionResourceLease::serial(LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(
                std::num::NonZeroUsize::MIN,
                Some(64 * 1024 * 1024),
                Some(host.request_work),
            ),
        ),
        cancellation: scope.cancellation().execution_token(),
        deadline: scope.deadline(),
    })
    .expect("the finite caller request is admitted");
    host.program
        .runtime()
        .with_funded_application_advancement(
            scope,
            ExecutionAllocationPolicy::Execution(&caller),
            body,
        )
        .expect("the installed phase borrows the original funded caller")
}
