use worth_runtime_bridge::facade::{
    BridgeSubscriptionCertificationCostPostureReport,
    BridgeSubscriptionCertificationCounterSnapshot,
};


fn main() {
    let _report = BridgeSubscriptionCertificationCostPostureReport {
        sparse_cost_profile_digest: sealed_authority_placeholder(),
        dense_cost_profile_digest: sealed_authority_placeholder(),
        over_budget_rejection_digest: sealed_authority_placeholder(),
        first_scratch_digest: sealed_authority_placeholder(),
        repeated_scratch_digest: sealed_authority_placeholder(),
        dense_selected_before_assembly: true,
        over_budget_rejected_before_assembly: true,
        scratch_lifecycle_reuse_visible: true,
        counters: BridgeSubscriptionCertificationCounterSnapshot::default(),
        canonical_basis: sealed_authority_placeholder(),
        digest: sealed_authority_placeholder(),
    };
}

fn sealed_authority_placeholder<T>() -> T {
    panic!("compile-fail fixture never executes")
}
