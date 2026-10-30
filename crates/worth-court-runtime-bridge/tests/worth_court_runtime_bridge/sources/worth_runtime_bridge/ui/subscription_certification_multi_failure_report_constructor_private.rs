use worth_runtime_bridge::facade::{
    BridgeSubscriptionCertificationCounterSnapshot,
    BridgeSubscriptionCertificationFailureBoundary,
    BridgeSubscriptionCertificationFailurePrecedenceStage,
    BridgeSubscriptionCertificationMultiFailurePrecedenceReport,
};

fn main() {
    let _report = BridgeSubscriptionCertificationMultiFailurePrecedenceReport {
        control_bundle_digest: sealed_authority_placeholder(),
        hostile_bundle_digest: sealed_authority_placeholder(),
        comparison_report_digest: sealed_authority_placeholder(),
        primary_failure_boundary: BridgeSubscriptionCertificationFailureBoundary::BasisDrift,
        primary_failure_precedence_stage:
            BridgeSubscriptionCertificationFailurePrecedenceStage::BasisBinding,
        suppressed_failure_boundaries: Vec::new(),
        basis_drift_is_primary_without_registry_drift: true,
        suppressed_checkpoint_replay_and_diagnostics: true,
        counters: BridgeSubscriptionCertificationCounterSnapshot::default(),
        canonical_basis: sealed_authority_placeholder(),
        digest: sealed_authority_placeholder(),
    };
}

fn sealed_authority_placeholder<T>() -> T {
    panic!("compile-fail fixture never executes")
}
