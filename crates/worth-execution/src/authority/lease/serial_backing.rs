//! An owned, policy-bounded lease backing that has no physical pool.
use super::{ExecutionResourceLease, LeaseDenial, LeaseRequest};
use std::num::NonZeroUsize;
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

impl ExecutionResourceLease<'static> {
    /// Grants a serial request the same checked execution doors as a physical
    /// lease. The backing owns no pool and never constructs a thread. Children
    /// share its ledger and inherit its serial posture, interruption and limits.
    /// An equivalence contract needs a physical authority's installed registry.
    pub fn serial(request: LeaseRequest) -> Result<Self, LeaseDenial> {
        if !matches!(
            request.policy.determinism(),
            DeterminismContract::CanonicalBitwise
        ) {
            return Err(LeaseDenial::refused(
                LeaseDenial::EquivalenceContractUnavailable,
            ));
        }
        let budget = request.policy.budget();
        let policy = ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            request.policy.determinism(),
            ExecutionBudget::new(
                NonZeroUsize::MIN,
                budget.charged_memory_bytes(),
                budget.work_ceiling(),
            ),
        );
        super::ExecutionAuthority::serial().request_lease(LeaseRequest {
            policy,
            cancellation: request.cancellation,
            deadline: request.deadline,
        })
    }
}

#[cfg(test)]
mod tests;
