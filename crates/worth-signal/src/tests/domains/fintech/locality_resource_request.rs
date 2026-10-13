//! Resources authored by the fixture caller and admitted by the real owners.
use crate::facade::SignalRuntimePolicy;
use worth_foundational::facade::CanonicalDigestWorkBudget;

#[derive(Clone, Copy)]
pub(crate) struct FinancialLocalityResourceRequest {
    pub policy: SignalRuntimePolicy,
    pub maximum_work: u64,
    pub wave_memory_bytes: u64,
    pub churn_memory_bytes: u64,
    pub digest_budget: CanonicalDigestWorkBudget,
}

impl FinancialLocalityResourceRequest {
    pub(crate) fn ordinary(policy: SignalRuntimePolicy) -> Self {
        Self {
            policy,
            maximum_work: 100_000_000,
            wave_memory_bytes: 128 * 1024 * 1024,
            churn_memory_bytes: 64 * 1024 * 1024,
            digest_budget: CanonicalDigestWorkBudget::standard(),
        }
    }
}

impl From<SignalRuntimePolicy> for FinancialLocalityResourceRequest {
    fn from(policy: SignalRuntimePolicy) -> Self {
        Self::ordinary(policy)
    }
}
