//! Serial placement uses the same bounded lease path without a worker pool.
use super::*;

pub(super) fn form(
    policy: ExecutionRequestPolicy,
    cancellation: worth_execution::CancellationToken,
    deadline: Option<std::time::Instant>,
) -> Form<'static> {
    let policy = match policy.accounting() {
        Some(budget) => ExecutionRequestPolicy::new(
            worth_foundational::ExecutionPosture::Serial,
            policy.determinism(),
            worth_foundational::ExecutionBudget::new(
                std::num::NonZeroUsize::MIN,
                budget.charged_memory_bytes(),
                budget.work_ceiling(),
            ),
        ),
        None => ExecutionRequestPolicy::for_workers(
            worth_foundational::ExecutionPosture::Serial,
            policy.determinism(),
            std::num::NonZeroUsize::MIN,
        ),
    };
    Form::Serial {
        lease: worth_execution::ExecutionAuthority::serial().request_lease(LeaseRequest {
            policy,
            deadline,
            cancellation,
        }),
        policy,
    }
}
