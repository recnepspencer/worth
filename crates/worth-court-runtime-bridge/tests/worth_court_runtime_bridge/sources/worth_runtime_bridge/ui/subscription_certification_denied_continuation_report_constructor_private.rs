use worth_runtime_bridge::facade::{
    BridgeSubscriptionCertificationCounterSnapshot,
    BridgeSubscriptionCertificationDeniedContinuationReport,
    BridgeSubscriptionCertificationFailureBoundary,
    BridgeSubscriptionCertificationFailurePrecedenceStage,
};

fn main() {
    let _report = BridgeSubscriptionCertificationDeniedContinuationReport {
        admitted_bundle_digest: sealed_authority_placeholder(),
        denied_bundle_digest: sealed_authority_placeholder(),
        comparison_report_digest: sealed_authority_placeholder(),
        primary_failure_boundary:
            BridgeSubscriptionCertificationFailureBoundary::ContinuationDenialOrAmbiguity,
        primary_failure_precedence_stage:
            BridgeSubscriptionCertificationFailurePrecedenceStage::ContinuationOrBranchScope,
        denied_before_delivery_drift: true,
        counters: BridgeSubscriptionCertificationCounterSnapshot::default(),
        canonical_basis: sealed_authority_placeholder(),
        digest: sealed_authority_placeholder(),
    };
}

fn sealed_authority_placeholder<T>() -> T {
    panic!("compile-fail fixture never executes")
}
