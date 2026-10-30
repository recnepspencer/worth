use worth_runtime_bridge::facade::{
    BridgeSubscriptionCertificationCounterSnapshot, BridgeSubscriptionCertificationFailureBoundary,
    BridgeSubscriptionCertificationFailurePrecedenceStage,
    BridgeSubscriptionCertificationHistoricalBasisReport,
};

fn main() {
    let _report = BridgeSubscriptionCertificationHistoricalBasisReport {
        retained_basis_bundle_digest: sealed_authority_placeholder(),
        latest_unretained_bundle_digest: sealed_authority_placeholder(),
        comparison_report_digest: sealed_authority_placeholder(),
        primary_failure_boundary: BridgeSubscriptionCertificationFailureBoundary::BasisDrift,
        primary_failure_precedence_stage:
            BridgeSubscriptionCertificationFailurePrecedenceStage::BasisBinding,
        retained_basis_is_explicit: true,
        latest_truth_reconstruction_count: 0,
        counters: BridgeSubscriptionCertificationCounterSnapshot::default(),
        canonical_basis: sealed_authority_placeholder(),
        digest: sealed_authority_placeholder(),
    };
}

fn sealed_authority_placeholder<T>() -> T {
    panic!("compile-fail fixture never executes")
}
