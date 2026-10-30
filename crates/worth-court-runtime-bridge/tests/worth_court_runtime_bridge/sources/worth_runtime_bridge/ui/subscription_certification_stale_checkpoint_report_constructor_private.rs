use worth_runtime_bridge::facade::{
    BridgeSubscriptionCertificationCounterSnapshot,
    BridgeSubscriptionCertificationFailureBoundary,
    BridgeSubscriptionCertificationFailurePrecedenceStage,
    BridgeSubscriptionCertificationStaleCheckpointReport,
};

fn main() {
    let _report = BridgeSubscriptionCertificationStaleCheckpointReport {
        fresh_bundle_digest: sealed_authority_placeholder(),
        stale_bundle_digest: sealed_authority_placeholder(),
        fresh_checkpoint_digest: sealed_authority_placeholder(),
        stale_checkpoint_digest: sealed_authority_placeholder(),
        comparison_report_digest: sealed_authority_placeholder(),
        primary_failure_boundary: BridgeSubscriptionCertificationFailureBoundary::CheckpointDivergence,
        primary_failure_precedence_stage:
            BridgeSubscriptionCertificationFailurePrecedenceStage::CheckpointResumeOrReplay,
        checkpoint_drift_is_primary_without_replay_mismatch: true,
        suppressed_failure_boundary_count: 0,
        counters: BridgeSubscriptionCertificationCounterSnapshot::default(),
        canonical_basis: sealed_authority_placeholder(),
        digest: sealed_authority_placeholder(),
    };
}

fn sealed_authority_placeholder<T>() -> T {
    panic!("compile-fail fixture never executes")
}
