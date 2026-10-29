use worth_runtime_bridge::facade::{
    BridgeSubscriptionCertificationCounterSnapshot, BridgeSubscriptionCertificationFailureBoundary,
    BridgeSubscriptionCertificationFailurePrecedenceStage,
    BridgeSubscriptionCertificationStrategyLoweringReport,
};

fn main() {
    let _report = BridgeSubscriptionCertificationStrategyLoweringReport {
        detail_bundle_digest: sealed_authority_placeholder(),
        collection_bundle_digest: sealed_authority_placeholder(),
        comparison_report_digest: sealed_authority_placeholder(),
        primary_failure_boundary:
            BridgeSubscriptionCertificationFailureBoundary::StrategyLoweringProvenanceMismatch,
        primary_failure_precedence_stage:
            BridgeSubscriptionCertificationFailurePrecedenceStage::StrategyLowering,
        strategy_lowering_is_distinct_without_signal_rediscovery: true,
        counters: BridgeSubscriptionCertificationCounterSnapshot::default(),
        canonical_basis: sealed_authority_placeholder(),
        digest: sealed_authority_placeholder(),
    };
}

fn sealed_authority_placeholder<T>() -> T {
    panic!("compile-fail fixture never executes")
}
