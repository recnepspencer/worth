use worth_runtime_bridge::facade::BridgeSubscriptionOfflineAuditPlan;

fn main() {
    let _plan = BridgeSubscriptionOfflineAuditPlan {
        bundle_index_digest: sealed_authority_placeholder(),
        comparison_report_digests: Vec::new(),
        counters: Default::default(),
        canonical_basis: sealed_authority_placeholder(),
        digest: sealed_authority_placeholder(),
    };
}

fn sealed_authority_placeholder<T>() -> T {
    panic!("compile-fail fixture never executes")
}
