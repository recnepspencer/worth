use worth_runtime_bridge::facade::{
    BridgeSubscriptionCertificationComparisonOutcome,
    BridgeSubscriptionCertificationCounterSnapshot,
    BridgeSubscriptionCertificationOrderingHostilityReport,
};

fn main() {
    let _report = BridgeSubscriptionCertificationOrderingHostilityReport {
        control_source_artifact_index_digest: sealed_authority_placeholder(),
        hostile_source_artifact_index_digest: sealed_authority_placeholder(),
        control_bundle_digest: sealed_authority_placeholder(),
        hostile_bundle_digest: sealed_authority_placeholder(),
        comparison_report_digest: sealed_authority_placeholder(),
        comparison_outcome: BridgeSubscriptionCertificationComparisonOutcome::Equivalent,
        canonical_source_order_preserved: true,
        semantic_digest_preserved: true,
        sealed_bundle_digest_preserved: true,
        field_order_preserved: true,
        counters: BridgeSubscriptionCertificationCounterSnapshot::default(),
        canonical_basis: sealed_authority_placeholder(),
        digest: sealed_authority_placeholder(),
    };
}

fn sealed_authority_placeholder<T>() -> T {
    panic!("compile-fail fixture never executes")
}
