use worth_runtime_bridge::facade::{
    BridgeSubscriptionCertificationCounterSnapshot, BridgeSubscriptionCertificationFanoutReport,
};

fn main() {
    let _report = BridgeSubscriptionCertificationFanoutReport {
        shared_equivalence_report_digest: sealed_authority_placeholder(),
        divergent_rejection_report_digest: sealed_authority_placeholder(),
        shared_fanout_equivalent: true,
        divergent_sharing_rejected_before_delivery: true,
        counters: BridgeSubscriptionCertificationCounterSnapshot::default(),
        canonical_basis: sealed_authority_placeholder(),
        digest: sealed_authority_placeholder(),
    };
}

fn sealed_authority_placeholder<T>() -> T {
    panic!("compile-fail fixture never executes")
}
