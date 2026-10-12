//! The caller supplies the scheduled courtroom's resource request.
use crate::facade::SignalRuntimePolicy;
use crate::tests::domains::fintech::locality_resource_request::FinancialLocalityResourceRequest;
use worth_foundational::facade::CanonicalDigestWorkBudget;

pub(super) fn scheduled_request() -> FinancialLocalityResourceRequest {
    let outputs = 100_000_usize;
    let mut policy = SignalRuntimePolicy::operational()
        .with_maximum_waiter_resolution_visits(outputs * outputs * 16);
    policy.serial_memory_bytes = 512 * 1024 * 1024;
    FinancialLocalityResourceRequest {
        policy,
        maximum_work: 160_000_000_000,
        wave_memory_bytes: 128 * 1024 * 1024,
        churn_memory_bytes: 64 * 1024 * 1024,
        digest_budget: CanonicalDigestWorkBudget::new(4096, 32 * 1024 * 1024).unwrap(),
    }
}

pub(super) fn scheduled_run_request() -> FinancialLocalityResourceRequest {
    FinancialLocalityResourceRequest {
        wave_memory_bytes: 512 * 1024 * 1024,
        churn_memory_bytes: 512 * 1024 * 1024,
        ..scheduled_request()
    }
}
