use std::sync::{Arc, Mutex};

use worth_execution::{
    CancellationToken, ExecutionRequest, ExecutionResourceLease, ExecutionWorkCeiling,
    LeaseRequest, WorkCeilingDenial,
};
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
    /// Top-level packet runners and native evidence stages receive this account;
    /// kernels never receive it. A validated proposal carries the account into preparation.
    /// Hold the lock through each run so no dispatch borrows unreported work.
    pub(crate) fn run<R>(
        &self,
        lease: &ExecutionResourceLease<'_>,
        run: impl FnOnce(&ExecutionResourceLease<'_>) -> R,
        report: impl FnOnce(&R) -> ExecutionReport,
    ) -> R {
        self.run_reported(lease, run, |outcome| Some(report(outcome)))
    }

    /// Native evidence settles into the same account as packet preparation.
    /// A refusal before callback entry has no work report to settle.
    pub(crate) fn run_stage<'authority, R>(
        &self,
        lease: &ExecutionResourceLease<'authority>,
        stage: impl FnOnce(ExecutionRequest<'_, 'authority>) -> R,
    ) -> Result<R, WorkCeilingDenial> {
        self.run_reported(
            lease,
            |child| {
                let execution = ExecutionRequest::leased(child);
                execution.run(ExecutionWorkCeiling::new(None), |_| stage(execution))
            },
            |outcome| outcome.as_ref().ok().map(|(_, report)| *report),
        )
        .map(|(value, _)| value)
    }

    fn run_reported<'authority, R>(
        &self,
        lease: &ExecutionResourceLease<'authority>,
        run: impl FnOnce(&ExecutionResourceLease<'authority>) -> R,
        report: impl FnOnce(&R) -> Option<ExecutionReport>,
    ) -> R {
        let policy = *lease.policy();
        let Some(budget) = policy.accounting() else {
            return run(lease);
        };
        let mut charged = self
            .charged
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let remaining = budget
            .work_ceiling()
            .map(|ceiling| ceiling.saturating_sub(*charged));
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
                // None and a fresh token are safe: lease/limits.rs:58-62 retains the parent deadline and :99-108 checks parent cancellation.
                deadline: None,
                cancellation: CancellationToken::new(),
            })
            .expect(/* Proof: The child copies its already admitted parent policy and only reduces remaining work; it introduces no invalid limit or deadline relation. */ "a remaining-work child preserves its parent's policy limits");
        let outcome = run(&child);
        if let Some(report) = report(&outcome) {
            #[cfg(test)]
            observation::record(report.charged_work());
            *charged = charged.saturating_add(report.charged_work());
        }
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
        None => {
            let outcome = run(lease);
            #[cfg(test)]
            observation::record(report(&outcome).charged_work());
            outcome
        }
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
pub(crate) mod observation;
