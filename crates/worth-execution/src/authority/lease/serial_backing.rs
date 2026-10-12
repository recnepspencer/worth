//! An owned, policy-bounded lease backing that has no physical pool.
use super::{
    AuthorityInner, EquivalenceRegistry, ExecutionAuthorityConfig, ExecutionResourceLease,
    LeaseDenial, LeaseNode, LeaseRequest, Ledger,
};
use std::{
    num::NonZeroUsize,
    sync::{Arc, Mutex},
};
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
        let authority = Arc::new(AuthorityInner {
            config: ExecutionAuthorityConfig {
                max_workers: NonZeroUsize::MIN,
                charged_memory_bytes: None,
            },
            pool: None,
            equivalences: EquivalenceRegistry::default(),
            ledger: Mutex::new(Ledger::default()),
        });
        Ok(Self {
            node: Arc::new(LeaseNode {
                id: authority.next_id(),
                parent: None,
                max_workers: 1,
                posture: ExecutionPosture::Serial,
                charged_memory_bytes: budget.charged_memory_bytes(),
                deadline: request.deadline,
                cancellation: request.cancellation,
            }),
            authority,
            _authority: std::marker::PhantomData,
            policy,
        })
    }
}

#[cfg(test)]
mod tests;
