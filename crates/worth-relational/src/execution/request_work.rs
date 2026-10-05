use std::sync::{Arc, Mutex};

use worth_execution::{CancellationToken, ExecutionResourceLease, LeaseRequest};
use worth_foundational::{ExecutionBudget, ExecutionReport, ExecutionRequestPolicy};

/// The charged work of one leased validation or commit preparation request.
/// Clones are explicit references to the same request account.
#[derive(Debug, Clone, Default)]
pub(crate) struct RequestWorkBudget {
    charged: Arc<Mutex<u64>>,
}

impl RequestWorkBudget {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Validation and commit preparation dispatch these patterns in phase order.
    /// Only their top-level packet runners receive this account; kernels never
    /// receive it, and a recursive public request builds a fresh runtime snapshot.
    /// Hold the lock through each run so no dispatch borrows unreported work.
    pub(crate) fn run<R>(
        &self,
        lease: &ExecutionResourceLease<'_>,
        run: impl FnOnce(&ExecutionResourceLease<'_>) -> R,
        report: impl FnOnce(&R) -> ExecutionReport,
    ) -> R {
        let mut charged = self
            .charged
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let policy = *lease.policy();
        let budget = policy.budget();
        let remaining = budget.work_ceiling().saturating_sub(*charged);
        let child = lease
            .child(LeaseRequest {
                policy: ExecutionRequestPolicy::new(
                    policy.posture(),
                    policy.determinism(),
                    ExecutionBudget::new(
                        budget.max_workers(),
                        budget.charged_memory_bytes(),
                        remaining,
                    ),
                ),
                deadline: None,
                cancellation: CancellationToken::new(),
            })
            .expect("a remaining-work child preserves its parent's policy limits");
        let outcome = run(&child);
        *charged = charged.saturating_add(report(&outcome).charged_work());
        outcome
    }
}

pub(crate) fn run_with_remaining_request_work<R>(
    lease: &ExecutionResourceLease<'_>,
    budget: Option<&RequestWorkBudget>,
    run: impl FnOnce(&ExecutionResourceLease<'_>) -> R,
    report: impl FnOnce(&R) -> ExecutionReport,
) -> R {
    match budget {
        Some(budget) => budget.run(lease, run, report),
        None => run(lease),
    }
}
