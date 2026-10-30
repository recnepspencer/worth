use worth_runtime_bridge::facade::{
    BridgeSubscriptionOfflineAuditOutcome, BridgeSubscriptionOfflineAuditReport,
};

fn main() {
    let _report = BridgeSubscriptionOfflineAuditReport {
        outcome: BridgeSubscriptionOfflineAuditOutcome::DiagnosedOffline,
        bundle_index_digest: sealed_authority_placeholder(),
        comparison_report_count: 0,
        counters: Default::default(),
        canonical_basis: sealed_authority_placeholder(),
        digest: sealed_authority_placeholder(),
    };
}

fn sealed_authority_placeholder<T>() -> T {
    panic!("compile-fail fixture never executes")
}
