//! The caller supplies the scheduled courtroom's resource request.
use crate::facade::SignalRuntimePolicy;
use crate::tests::domains::fintech::locality_resource_request::FinancialLocalityResourceRequest;
use worth_foundational::facade::CanonicalDigestWorkBudget;

pub(super) fn scheduled_request() -> FinancialLocalityResourceRequest {
    // The 100,000-output roster required 306,628,328 preparation bytes:
    // round up to the next power of two (512 MiB), retaining 230,242,584
    // bytes of headroom. Wave/churn retain their ordinary 128/64 MiB here;
    // the full roster admits the same measured ceiling for those below.
    let outputs = 100_000_usize;
    // A finite ceiling of sixteen waiter visits per ordered output pair is
    // 100,000 * 100,000 * 16 = 160,000,000,000; work admits that same ceiling.
    // The visit count saturates at the machine word on 32-bit targets;
    // the separate u64 work amount retains its full ceiling.
    let mut policy = SignalRuntimePolicy::operational()
        .with_maximum_waiter_resolution_visits(outputs.saturating_mul(outputs).saturating_mul(16));
    policy.serial_memory_bytes = 512 * 1024 * 1024;
    FinancialLocalityResourceRequest {
        policy,
        maximum_work: 160_000_000_000,
        wave_memory_bytes: 128 * 1024 * 1024,
        churn_memory_bytes: 64 * 1024 * 1024,
        // Retain the standard 4096 digest entries. The scheduled digest exceeded
        // the ordinary 4 MiB byte budget; admit eight times that byte budget.
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
