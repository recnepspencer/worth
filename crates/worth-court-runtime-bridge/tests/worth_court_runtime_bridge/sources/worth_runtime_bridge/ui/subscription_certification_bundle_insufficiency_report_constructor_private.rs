use worth_runtime_bridge::facade::{
    BridgeSubscriptionCertificationBundleInsufficiencyReport,
    BridgeSubscriptionCertificationCounterSnapshot, BridgeSubscriptionCertificationFailureBoundary,
    BridgeSubscriptionCertificationFailurePrecedenceStage,
};

fn main() {
    let _report = BridgeSubscriptionCertificationBundleInsufficiencyReport {
        complete_bundle_digest: sealed_authority_placeholder(),
        insufficient_bundle_digest: sealed_authority_placeholder(),
        complete_completeness_report_digest: sealed_authority_placeholder(),
        insufficient_completeness_report_digest: sealed_authority_placeholder(),
        comparison_report_digest: sealed_authority_placeholder(),
        primary_failure_boundary: BridgeSubscriptionCertificationFailureBoundary::BundleInsufficiency,
        primary_failure_precedence_stage:
            BridgeSubscriptionCertificationFailurePrecedenceStage::RetainedArtifactCompleteness,
        insufficiency_is_primary_without_semantic_drift: true,
        counters: BridgeSubscriptionCertificationCounterSnapshot::default(),
        canonical_basis: sealed_authority_placeholder(),
        digest: sealed_authority_placeholder(),
    };
}

fn sealed_authority_placeholder<T>() -> T {
    panic!("compile-fail fixture never executes")
}
